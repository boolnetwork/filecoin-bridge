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
use bridge_primitives::{TssError,tss_error};
use async_trait::async_trait;

use crate::recover::recover;

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
            let result2 = self.tx_pool.submit_one(&at, source, xt);
            std::thread::spawn(|| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();
                let res = rt.block_on(result2);
                println!("======submit===result==={:?}",res);
            });
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
                    let message = sp_io::hashing::blake2_256(&payload[..]);
                    let url = self.tss_url();
                    let str_url = core::str::from_utf8(&url).unwrap();
                    let sig:Signature = match sign_by_tss(message.to_vec(),str_url,tss_gen_pubkey){
                        Ok(sig) => { ecdsa::Signature::from_slice(&sig).into() },
                        Err(e) => {
                            match tss_error(e){
                                TssError::SignUp() => {}, // do nothing
                                _ => {  recover();  }, // retry
                            }
                            ecdsa::Signature::default().into()
                        },
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
        }
    }
}
