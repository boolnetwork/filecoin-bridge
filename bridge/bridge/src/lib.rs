#[warn(dead_code)]
use std::{sync::Arc, u64, marker::PhantomData, thread};
use futures::{prelude::*, executor::block_on, channel::mpsc};
use parking_lot::Mutex;
use log::{debug, info};
use codec::{Encode, Decode};

use sp_runtime::{generic::{BlockId ,Era}, traits::{Block as BlockT, Zero}};
use sp_api::{ProvideRuntimeApi, CallApiAt};
use sp_core::{Pair, storage::{StorageKey, StorageData}, sr25519::Pair as edPair, sr25519, ecdsa, twox_128};
use sc_client_api::{BlockchainEvents, backend, notifications::StorageEventStream};
use sp_blockchain::{HeaderBackend};
use sp_transaction_pool::{TransactionPool, TransactionFor};
use sc_block_builder::{BlockBuilderProvider};
use node_primitives::Hash;
use num_traits::cast::FromPrimitive;
use frame_system::{Call as SystemCall, EventRecord};
use pallet_tss::{Call as TssCall, RawEvent, WithdrawDetail };

use filecoin_bridge_runtime::{UncheckedExtrinsic, apis::VendorApi ,Call, SignedPayload
							  , Event, VERSION, Runtime, AccountId, Signature, Balance, Index};

use tss_signer::{set_pubkey, sign_btc_hex_return_hex, sign_by_tss};
use node_tss::{start_sm_manager, key_gen, push};

use lotus_api_forest::api::MpoolApi;
use forest_message;
use forest_cid;
use forest_vm::{self, Serialized};
use forest_encoding::Cbor;
use forest_crypto;

use async_trait::async_trait;
use bridge_primitives::{TssError,tss_error};

mod chainstate;
pub use chainstate::ChainState;

mod txsender;
pub use txsender::{TxSender, TxMessage, TxType, SuperviseClient, PacketNonce, TokenType};

mod filecoinapi;
pub use filecoinapi::{get_nonce, message_create, send_fc_message};

mod recover;
pub use recover::recover;

pub enum TssRole{
	Manager,
	Party,
}

pub enum SignatureType{
	Btc,
	General,
}

trait PrefixKey {
	fn as_prefix_key(&self) -> Vec<u8>;
}

impl PrefixKey for [u8] {
	fn as_prefix_key(&self) -> Vec<u8> {
		let mut key = [0u8;32];
		let mut items = self.split(|spa| *spa == b' ');
		if let Some(module) = items.next() {
			key[0..16].copy_from_slice(&twox_128(module));
		}
		if let Some(name) = items.next() {
			key[16..].copy_from_slice(&twox_128(name));
		}
		key.to_vec()
	}
}

#[derive(Debug, Clone)]
pub struct TssSender<V,B> {
	pub spv: V,
	pub tss: u64,
    pub senderbool: FcPubkeySender,
	pub senderfc: FcPubkeySender,
	pub a: std::marker::PhantomData<B>,
}

impl <V,B>TssSender<V,B>
	where   V: SuperviseClient<B> + Send + Sync + 'static,
			B: BlockT,
{
	pub fn new(spv: V,senderb:FcPubkeySender,senderfc:FcPubkeySender) -> Self {
		TssSender {
			spv: spv,
			tss: 5,
			senderbool: senderb,
			senderfc: senderfc,
			a: PhantomData,
		}
	}

	fn submit_tx(&self,data:TxMessage) {
         self.spv.submit(data);
	}

	fn submit_tx_ecdsa(&self) {
		self.spv.submit_fc_transfer_tss(TxMessage::new(TxType::System));
	}

	fn key_gen(&self,url:Vec<u8>,_store:Vec<u8>){
		let str_url = core::str::from_utf8(&url).unwrap();
		let store2 = "boolbtc.store";
        match key_gen(str_url,store2){
			Ok((pk,pk_vec)) => {
				let data = TxMessage::new(TxType::TssKeyGen(pk.to_vec(),pk_vec));
				println!("=========key_gen===submit_tx=1==");
				self.submit_tx(data);
				//println!("=========key_gen===submit_tx=2==");
				//let data2 = TxMessage::new(TxType::BtcAddressSet(pk.to_vec()));
				//self.submit_tx(data2);
				//set_pubkey(pk.to_vec(),store2);
			},
			_ => return ,
		}
	}

	fn key_gen_bool(&self,url:Vec<u8>,_store:Vec<u8>){
		let str_url = core::str::from_utf8(&url).unwrap();
		let store2 = "bool.store";
		match key_gen(str_url,store2){
			Ok((pk,pk_vec)) => {
				let data = TxMessage::new(TxType::TssKeyGenBool(pk.to_vec(),pk_vec));
				self.submit_tx(data);
			},
			_ => return ,
		}
	}

	fn key_gen_fc(&self,url:Vec<u8>,_store:Vec<u8>){
		let str_url = core::str::from_utf8(&url).unwrap();
		let store2 = "filecoin.store";
		match key_gen(str_url,store2){
			Ok((pk,pk_vec)) => {
				let data = TxMessage::new(TxType::TssKeyGenFc(pk.to_vec(),pk_vec));
				self.submit_tx(data);
			},
			_ => return ,
		}
	}

	fn key_sign(&self,url:Vec<u8>, message:Vec<u8>, pubkey:Vec<u8> ,sigtype:SignatureType) -> Option<Vec<u8>>{
		let str_url = core::str::from_utf8(&url).unwrap();
		//let pubkey = self.spv.tss_pubkey();
		debug!(target:"keysign", "pubkey {:?}", pubkey);
		match sigtype {
			SignatureType::General => {
				let _str_message = core::str::from_utf8(&message).unwrap();
				let signature = sign_by_tss(message, str_url,pubkey).unwrap();
				return Some(signature)
			},
			SignatureType::Btc => {
				let pubkey = self.spv.tss_pubkey();
				debug!(target:"keysign", "SignatureType::Btc");
				let res = sign_btc_hex_return_hex(message,str_url,pubkey);
				if res.is_ok(){
					let signed_tx = res.unwrap();
					let data = TxMessage::new(TxType::Signature(signed_tx.into()));
					self.submit_tx(data);
					return None
				   }
				}
	        }
		Some(vec![0u8]) // None?
	}

	fn withdraw_fc(&self, withdrawdetail:&WithdrawDetail<AccountId>){
		let url = self.spv.tss_url();
		let pubkey = self.spv.tss_pubkey();
		let (message,cid) = message_create(
			pubkey.clone(),
			withdrawdetail.receiver.clone(),
			withdrawdetail.value.clone(),
		);
		let sig = self.key_sign(url, cid.to_bytes(),
								pubkey.clone(),SignatureType::General).unwrap();
		let signed_message = forest_message::SignedMessage{
			message:message,
			signature:forest_crypto::Signature::new_secp256k1(sig),
		};
		let cid = send_fc_message(signed_message);
		println!("withdraw fc result ----------> {:?}",cid);
	}

	fn get_stream(&self, events_key:StorageKey) -> StorageEventStream<B::Hash> {
		self.spv.get_notification_stream(Some(&[events_key]), None)
	}

	pub fn start(self,
				 _role: TssRole,
				 enable_tss_message_intermediary: bool ) -> impl Future<Output=()> + 'static {

		let events_key = StorageKey(b"System Events".as_prefix_key());
		let storage_stream: StorageEventStream<B::Hash> = self.get_stream(events_key);

		let storage_stream = storage_stream
			.for_each( move|(_blockhash,change_set)| {
				let records: Vec<Vec<EventRecord<Event, Hash>>> = change_set
					.iter()
					.filter_map(|(_ , _, mbdata)| {
						if let Some(StorageData(data)) = mbdata {
							Decode::decode(&mut &data[..]).ok()
						} else {
							None
						}
					})
					.collect();
				let events: Vec<Event> = records.concat().iter().cloned().map(|r| r.event).collect();
				events.iter().for_each(|event| {
					debug!(target:"keysign", "Event {:?}", event);
					if enable_tss_message_intermediary {
						if let Event::pallet_tss(e) = event {
							match e {
								RawEvent::GenKey(_index, _id, _time, url) => {
									self.key_gen(url.to_vec(), vec![0u8]);
								},
								RawEvent::GenerateTssKey(url, store) => {
									self.key_gen(url.to_vec(), store.to_vec());
								},
								RawEvent::GenerateTssKeyBool(url, store) => {
									self.key_gen_bool(url.to_vec(), store.to_vec());
								},
								RawEvent::GenerateTssKeyFc(url, store) => {
									self.key_gen_fc(url.to_vec(), store.to_vec());
								},
								RawEvent::SignMessage(_index, _id, _time, url, message, pubkey) => {
									self.key_sign(url.to_vec(), message.to_vec(), pubkey.to_vec() ,SignatureType::General);
								},
								RawEvent::SignBtcMessage(_index, _time, url, message, pubkey) => {
									self.key_sign(url.to_vec(), message.to_vec(), pubkey.to_vec() ,SignatureType::Btc);
								},
								RawEvent::WithdrawToken(withdrawdetail) => {
								    self.withdraw_fc(withdrawdetail);
   							    },
								_ => {}
							}
						}
					}
				});
				futures::future::ready(())
			});

		println!("=========== return storage_stream ===========");
		storage_stream
	}
}


type FcPubkeySender = mpsc::UnboundedSender<Vec<u8>>;

pub fn start_tss<A, B, C, Block>(
	client: Arc<C>,
	pool: Arc<A>,
	_keystore: u64,
	enable_tss_message_intermediary:bool,
	senderbool: FcPubkeySender,
	senderfc: FcPubkeySender,
	num:u64,
) -> impl Future<Output = ()> + 'static
	where
		A: TransactionPool<Block = Block> + 'static,
		Block: BlockT,
		B: backend::Backend<Block> + Send + Sync + 'static,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
		C::Api: VendorApi<Block>,
		Block::Hash: Into<sp_core::H256>
{

	let _sign_key = "2f2f416c69636508080808080808080808080808080808080808080808080808".to_string();
	let key = sr25519::Pair::from_string(&format!("//{}", "Eve"), None)
		.expect("static values are valid; qed");

	//let key_seed = sr25519::Pair::from_seed_slice(&[0x25,0xb4,0xfd,0x88,0x81,0x3f,0x5e,0x16,0xd4,0xbe,0xa6,0x28
	//	,0x39,0x02,0x89,0x57,0xf9,0xe3,0x40,0x10,0x8e,0x4e,0x93,0x73,0xd0,0x8b,0x31,0xb0,0xf6,0xe3,0x04,0x40]).unwrap();

	let at = BlockId::Hash(client.info().best_hash);
	let tx_sender = TxSender::new(
		client,
		pool,
		key,
		Arc::new(parking_lot::Mutex::new(PacketNonce {nonce:0,last_block:at})),
	);

	let tss_sender = TssSender::new(
		tx_sender,
		senderbool,
		senderfc
	);

	if enable_tss_message_intermediary{
		thread::spawn(move || {
			start_sm_manager(num);
		});
	}else{
		push(num);
	}
	tss_sender.start(TssRole::Party ,!enable_tss_message_intermediary /*, on_exit*/)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn g() {

	}
}