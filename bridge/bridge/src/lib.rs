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

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
	FC,
	Bool,
}

impl TokenType{
	fn to_vec(&self) -> Vec<u8>{
		match self {
			TokenType::FC => vec![01u8,02u8,03u8],
			TokenType::Bool => vec![21u8,02u8,03u8],
		}
	}
}
#[derive(Debug, Clone, PartialEq)]
pub enum TxType {
	Spv,
	System,
	TssKeyGen(Vec<u8>,Vec<Vec<u8>>),
	TssKeyGenBool(Vec<u8>,Vec<Vec<u8>>),
	TssKeyGenFc(Vec<u8>,Vec<Vec<u8>>),

	BtcAddressSet(Vec<u8>),
	Signature(Vec<u8>),

	BoolDeposit(Vec<u8>,TokenType,u64), //who tokentype amount
	FCDeposit(Vec<u8>,TokenType,u128, Vec<u8>), //who tokentype amount from

	// TssKeyActive
	TssKeyGenActive(Vec<u8>,Vec<u8>),
	TssKeyGenBoolActive(Vec<u8>,Vec<u8>),
	TssKeyGenFcActive(Vec<u8>,Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct TxMessage {
	/// The type of Message.
	pub tx_type: TxType,
}

impl TxMessage{
	pub fn new(data:TxType) -> Self{
		TxMessage{
			tx_type:data
		}
	}
}
#[async_trait]
pub trait SuperviseClient<B>
	where
		B:BlockT
{
	fn get_notification_stream(&self,filter_keys: Option<&[StorageKey]>,
							   child_filter_keys: Option<&[(StorageKey, Option<Vec<StorageKey>>)]>) -> StorageEventStream<B::Hash>;
	fn is_tss_party(&self) -> bool;

	fn tss_pubkey(&self) -> Vec<u8>;
	fn tss_pubkey_bool(&self) -> Vec<u8>;
	fn tss_pubkey_fc(&self) -> Vec<u8>;
	fn tss_url(&self) -> Vec<u8>;

	fn submit(&self, message: TxMessage);
	fn submit_fc_transfer_tss(&self, message: TxMessage);

	fn submit_key_gen_bool_tss(&self);
}

pub struct PacketNonce<B>
	where
		B: BlockT,
{
	pub nonce: u64, // to control nonce.
    pub last_block: BlockId<B>,
}

impl <B>PacketNonce<B>
	where
		B: BlockT,
{
    pub fn new() -> PacketNonce<B>{
		PacketNonce{
			nonce:0,
			last_block: BlockId::number(0.into()),
		}
	}
}

#[derive(Clone)]
pub struct ChainState<Block,B,C>
	where
		Block: BlockT,
		B: backend::Backend<Block> + Send + Sync + 'static,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
 		C::Api: VendorApi<Block>,
{
	pub client: Arc<C>,
	_phantom: PhantomData<(Block,B)>,
}

impl <Block,B,C>ChainState<Block,B,C>
    where
		Block: BlockT,
		B: backend::Backend<Block> + Send + Sync + 'static,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
 		C::Api: VendorApi<Block>,
{
	pub fn new(client:Arc<C>) -> Self{
		ChainState{
			client:client,
			_phantom: PhantomData,
		}
	}

	pub fn tss_pubkey(&self) -> Vec<u8> {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		self.client
			.runtime_api()
			.tss_pub_key(&at)
			.unwrap()
	}
}

#[derive(Clone)]
pub struct TxSender<A,Block,B,C>
	where
		A: TransactionPool<Block = Block> + 'static,
		Block: BlockT,
		B: backend::Backend<Block> + Send + Sync + 'static,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
		C::Api: VendorApi<Block>,
		Block::Hash: Into<sp_core::H256>
{
	pub client: Arc<C>,
	pub tx_pool: Arc<A>,
	pub ed_key: edPair,
	pub packet_nonce: Arc<Mutex<PacketNonce<Block>>>,
	_phantom: PhantomData<B>,
}

impl<A,Block,B,C> TxSender<A,Block,B,C>
	where
		A: TransactionPool<Block = Block> + 'static,
		B: backend::Backend<Block> + Send + Sync + 'static,
		Block: BlockT,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
		C::Api: VendorApi<Block>,
		Block::Hash: Into<sp_core::H256>
{
	pub fn new(client:Arc<C>,tx_pool:Arc<A> /*,key:KeyStorePtr*/
			   ,ed_key:edPair,packet_nonce:Arc<Mutex<PacketNonce<Block>>>) -> Self{
		TxSender{
			client:client,
			tx_pool:tx_pool,
			ed_key: ed_key,
			packet_nonce:packet_nonce,
			_phantom: PhantomData,
		}
	}

	fn get_nonce(&self) -> u64 {
		let mut p_nonce = self.packet_nonce.lock();
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		if p_nonce.last_block == at {
			p_nonce.nonce = p_nonce.nonce + 1;
		} else {
			p_nonce.nonce = self
				.client
				.runtime_api()
				.account_nonce(&at, &self.ed_key.public().0.into())
				.unwrap();
			p_nonce.last_block = at;
		}

		p_nonce.nonce
	}

	fn get_nonce_any(&self,accountid:[u8;32]) -> u64 {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);
		self.client.runtime_api().account_nonce(&at, &accountid.into()).unwrap()
	}

}

impl<A,Block,B,C> SuperviseClient<Block> for TxSender<A,Block,B,C>
	where
		A: TransactionPool<Block = Block> + 'static,
		Block: BlockT,
		B: backend::Backend<Block> + Send + Sync + 'static,
		C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
		+ CallApiAt<Block> + Send + Sync + 'static,
		C::Api: VendorApi<Block>,
		Block::Hash: Into<sp_core::H256>
{
	fn get_notification_stream(&self,filter_keys: Option<&[StorageKey]>,
							   child_filter_keys: Option<&[(StorageKey, Option<Vec<StorageKey>>)]>) -> StorageEventStream<Block::Hash> {
		self.client.storage_changes_notification_stream(filter_keys,child_filter_keys)
			.unwrap()
	}

	fn is_tss_party(&self) -> bool {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

	    self.client
		.runtime_api()
		.is_tss_party(&at, &self.ed_key.public().0.into())
		.unwrap()
	}

	fn tss_pubkey(&self) -> Vec<u8> {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		self.client
			.runtime_api()
			.tss_pub_key(&at)
			.unwrap()
	}

	fn tss_pubkey_bool(&self) -> Vec<u8> {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		self.client
			.runtime_api()
			.tss_pub_key_bool(&at)
			.unwrap()
	}

	fn tss_pubkey_fc(&self) -> Vec<u8> {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		self.client
			.runtime_api()
			.tss_pub_key_fc(&at)
			.unwrap()
	}

	fn tss_url(&self) -> Vec<u8> {
		let info = self.client.info();
		let at: BlockId<Block> = BlockId::Hash(info.best_hash);

		self.client
			.runtime_api()
			.tss_url(&at)
			.unwrap()
	}

	fn submit(&self, relay_message: TxMessage) {
		let local_id: AccountId = self.ed_key.public().0.into();
		let info = self.client.info();
		let at = BlockId::Hash(info.best_hash);
		{
			let nonce = self.get_nonce();

			let function = match relay_message.tx_type {
				TxType::System => Call::System(SystemCall::remark(vec![1u8])),
				TxType::TssKeyGen(tss_pubkey,pk_vec) => Call::Tss(TssCall::key_created_result_is(tss_pubkey,pk_vec,vec![0u8])),
				TxType::TssKeyGenBool(tss_pubkey,pk_vec) => {
					let tss_gen_pubkey = tss_pubkey.clone(); // u8 65
					let publickey = secp256k1::PublicKey::parse_slice(&tss_gen_pubkey,None).unwrap();
					let compressed_pubkey = publickey.serialize_compressed();
					let pubkey_blake = sp_io::hashing::blake2_256(&compressed_pubkey[..]);
					let local_id:AccountId = pubkey_blake.into();
					println!("========bool_accountid========{:?}",local_id);
					Call::Tss(TssCall::key_created_result_is_bool(tss_pubkey,pk_vec,vec![0u8]))},
				TxType::TssKeyGenFc(tss_pubkey,pk_vec) => Call::Tss(TssCall::key_created_result_is_fc(tss_pubkey,pk_vec,vec![0u8])),
				//active
				TxType::TssKeyGenActive(url,store) => Call::Tss(TssCall::key_gen(url,store)),
				TxType::TssKeyGenBoolActive(url,store) => Call::Tss(TssCall::key_gen_bool(url,store)),
				TxType::TssKeyGenFcActive(url,store) => Call::Tss(TssCall::key_gen_fc(url,store)),

				//TxType::BtcAddressSet(tss_pubkey) => Call::BtcBridge(BtcBridgeCall::set_tss_revice_address(tss_pubkey)),
				//TxType::Signature(signed_btc_tx) => Call::BtcBridge(BtcBridgeCall::put_signedbtctxproposal(signed_btc_tx)),
				_ => Call::System(SystemCall::remark(vec![1u8])),
			};

			let extra = |i: Index, f: Balance| {
				(
					frame_system::CheckSpecVersion::<Runtime>::new(),
					frame_system::CheckTxVersion::<Runtime>::new(),
					frame_system::CheckGenesis::<Runtime>::new(),
					frame_system::CheckEra::<Runtime>::from(Era::Immortal),
					frame_system::CheckNonce::<Runtime>::from(i),
					frame_system::CheckWeight::<Runtime>::new(),
					pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(f),
					//Default::default(),
				)
			};
			let _version = self.client.runtime_version_at(&at).unwrap().spec_version;
			let genesis_hash = self.client.hash(Zero::zero())
				.expect("Genesis block always exists; qed").unwrap().into();
			let raw_payload = SignedPayload::from_raw(
				function,
				extra(nonce as u32 , 0),
				(
					VERSION.spec_version,
					VERSION.transaction_version,
					genesis_hash,
					genesis_hash,
					(),
					(),
					(),
				),
			);
			let signature = raw_payload.using_encoded(|payload| self.ed_key.sign(payload));
			let (function, extra, _) = raw_payload.deconstruct();
			let extrinsic =
				UncheckedExtrinsic::new_signed(function, local_id.into(), signature.into(), extra);
			let xt: TransactionFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
			debug!(target:"witness", "extrinsic {:?}", xt);
			let source = sp_runtime::transaction_validity::TransactionSource::External;
			//let result = block_on(self.tx_pool.submit_one(&at, source.clone(), xt));
			let result2 = self.tx_pool.submit_one(&at, source, xt);
			std::thread::spawn(|| {
				let mut rt = tokio::runtime::Runtime::new().unwrap();
				let res = rt.block_on(result2);
				println!("======submit===result==={:?}",res);
			});
			//info!("SuperviseClient submit transaction {:?}", result);
		}
	}

	fn submit_key_gen_bool_tss(&self){
		let url:Vec<u8> = vec![104u8, 116, 116, 112, 58, 47, 47, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 56, 48, 48, 49];
		let store:Vec<u8> = vec![102u8, 105, 108, 101, 99, 111, 105, 110, 46, 115, 116, 111, 114, 101];
		let data1:TxMessage = TxMessage::new(TxType::TssKeyGenActive(url.clone(),store.clone()));
		let data2:TxMessage = TxMessage::new(TxType::TssKeyGenBoolActive(url.clone(),store.clone()));
		let data3:TxMessage = TxMessage::new(TxType::TssKeyGenFcActive(url.clone(),store.clone()));
        self.submit(data1);
	}

	fn submit_fc_transfer_tss(&self, relay_message: TxMessage) {
 		let tss_gen_pubkey = self.tss_pubkey_bool(); // u8 65
		let publickey = secp256k1::PublicKey::parse_slice(&tss_gen_pubkey,None).unwrap();
		let compressed_pubkey = publickey.serialize_compressed();
		let pubkey_blake = sp_io::hashing::blake2_256(&compressed_pubkey[..]);
		let local_id:AccountId = pubkey_blake.into();
		println!("local_id={:?}",local_id);

		let info = self.client.info();
		let at = BlockId::Hash(info.best_hash);
		{
			let nonce = self.get_nonce_any(pubkey_blake.into());

			let function = match relay_message.tx_type {
				TxType::FCDeposit(who,_tokentype, value, from) =>
					Call::Tss(TssCall::deposit_token(who, value, from)),
				_ => Call::System(SystemCall::remark(vec![1u8])),
			};

			let extra = |i: Index, f: Balance| {
				(
					frame_system::CheckSpecVersion::<Runtime>::new(),
					frame_system::CheckTxVersion::<Runtime>::new(),
					frame_system::CheckGenesis::<Runtime>::new(),
					frame_system::CheckEra::<Runtime>::from(Era::Immortal),
					frame_system::CheckNonce::<Runtime>::from(i),
					frame_system::CheckWeight::<Runtime>::new(),
					pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(f),
				)
			};
			let _version = self.client.runtime_version_at(&at).unwrap().spec_version;
			let genesis_hash = self.client.hash(Zero::zero())
				.expect("Genesis block always exists; qed").unwrap().into();
			let raw_payload = SignedPayload::from_raw(
				function,
				extra(nonce as u32 , 0),
				(
					VERSION.spec_version,
					VERSION.transaction_version,
					genesis_hash,
					genesis_hash,
					(),
					(),
					(),
				),
			);
//			let key = ecdsa::Pair::from_string(&format!("//{}", "Eve"), None).expect("static values are valid; qed");

			let signature:Signature = raw_payload.using_encoded(|payload|
				{
					let mut message = payload;
                    if payload.len() != 32usize {
						let message = sp_io::hashing::blake2_256(&payload[..]);
					}
					let url = self.tss_url();
					let str_url = core::str::from_utf8(&url).unwrap();
					let sig:Signature = match sign_by_tss(message.to_vec(),str_url,tss_gen_pubkey){
						Ok(sig) => { ecdsa::Signature::from_slice(&sig).into() },
						Err(_) => { ecdsa::Signature::default().into() },
					};
					sig
				});
			if signature == ecdsa::Signature::default().into(){
				return;
			}
			let (function, extra, _) = raw_payload.deconstruct();

			let extrinsic =
				UncheckedExtrinsic::new_signed(function, local_id.into(), signature.into(), extra);
			let xt: TransactionFor<A> = Decode::decode(&mut &extrinsic.encode()[..]).unwrap();
			let source = sp_runtime::transaction_validity::TransactionSource::External;
			let result2 = self.tx_pool.submit_one(&at, source, xt);
			std::thread::spawn(|| {
				let mut rt = tokio::runtime::Runtime::new().unwrap();
				let res = rt.block_on(result2);
				info!("==========FileCoin Deposit========== {:?}", res);
			});
			//info!("SuperviseClient submit transaction {:?}", result);
		}
	}

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

pub enum TssRole{
	Manager,
	Party,
}

pub enum SignatureType{
	Btc,
	General,
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
		Some(vec![0u8])
	}

	fn withdraw_fc(&self, withdrawdetail:&WithdrawDetail<AccountId>){
		let url = self.spv.tss_url();
		let pubkey = self.spv.tss_pubkey_fc();
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
									//self.submit_tx_ecdsa();
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
	_keystore: u64/*KeyStorePtr*/,
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

use tokio::runtime::Runtime as tokioRuntime;

pub fn get_nonce(addr: forest_address::Address) -> u64 {
	let mut rt = tokioRuntime::new().unwrap();
	let http = lotus_api_forest::Http::new("http://127.0.0.1:1234/rpc/v0");
	let ret:u64 = rt.block_on(http.mpool_get_nonce(&addr)).unwrap();
	ret
}

pub fn message_create(from:Vec<u8>,to:Vec<u8>,val:u128,) -> (forest_message::UnsignedMessage ,forest_cid::Cid){
	let from_addr = forest_address::Address::new_secp256k1(&from).unwrap();
	let nonce = get_nonce(from_addr.clone());
	let unsignedtx = forest_message::UnsignedMessage {
		version: 0,
		to: forest_address::Address::new_bls(&to).unwrap(),
		from: from_addr,
    	sequence: nonce,
		value: forest_vm::TokenAmount::from_u128(val).unwrap(),
		method_num: 0u64,
		params: Serialized::new(vec![0u8]),
		gas_limit: 100000i64,
		gas_fee_cap:forest_vm::TokenAmount::from_u128(100000u128).unwrap(),
	    gas_premium:forest_vm::TokenAmount::from_u128(100000u128).unwrap(),
	};
	(unsignedtx.clone(),unsignedtx.cid().unwrap())
}

fn send_fc_message(message: forest_message::SignedMessage) -> forest_cid::Cid {
	let mut rt = tokioRuntime::new().unwrap();
	let http = lotus_api_forest::Http::new("http://127.0.0.1:1234/rpc/v0");
	let ret = rt.block_on(http.mpool_push(&message)).unwrap();
	ret
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn g() {

	}
}