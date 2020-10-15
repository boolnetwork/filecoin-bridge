#![allow(dead_code)]
use serialization::bytes::Bytes;
use serialization::{Serializable, Stream};

use libsecp256k1::PublicKey;
#[macro_use]
extern crate lazy_static;
use node_tss::sign_vec;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
use serialization::deserialize;
use std::collections::HashMap;

use libsecp256k1::Signature as secpSignature;
use secp256k1::key::SecretKey;

use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use curv::elliptic::curves::traits::ECScalar;
use curv::FE;

use log::{debug, info};
use lotus_api::types::message::UnsignedMessage;
use num_traits::cast::ToPrimitive;

use futures::executor::block_on;
use futures::{channel::mpsc, prelude::*};

use bridge::{PacketNonce, SuperviseClient, TokenType, TxMessage, TxSender, TxType};
use sp_transaction_pool::{TransactionFor, TransactionPool};
use std::marker::PhantomData;

use filecoin_bridge_runtime::apis::VendorApi;
use sc_block_builder::BlockBuilderProvider;
use sc_client_api::backend;
use sc_client_api::BlockchainEvents;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::sr25519;
use sp_core::sr25519::Pair as edPair;
use sp_core::Pair;
use sp_runtime::{
    generic::{BlockId, Era},
    traits::{Block as BlockT, Zero},
};
use std::fs::remove_dir;
use std::thread;

use lotus_api::{
    api::ChainApi, types::address::Address, types::message::BlockMessages, types::tipset::TipSet,
    Http,
};
use std::time;
use tokio::runtime::Runtime;

use lotus_api::types::message::originAddress;
use std::collections::HashSet;

use forest_address;
use forest_bigint;
use forest_cid;
use forest_message;
use interpreter;

lazy_static! {
    pub static ref STORE_LIST: Mutex<Vec<&'static str>> = Mutex::new(vec!["test"]);
}

#[derive(Debug)]
pub struct FCSender<V, B> {
    pub spv: V,
    pub reciver: MessageStreamR<Value>,
    pub a: std::marker::PhantomData<B>,
}

impl<V, B> FCSender<V, B>
where
    V: SuperviseClient<B> + Send + Sync + 'static,
    B: BlockT,
{
    pub fn new(spv: V, rec: mpsc::UnboundedReceiver<(Vec<u8>, Value)>) -> Self {
        FCSender {
            spv: spv,
            reciver: rec,
            a: PhantomData,
        }
    }

    fn submit_tx(&self, data: TxMessage) {
        self.spv.submit_fc_transfer_tss(data);
    }

    fn sign_tss_message_start(self) -> impl Future<Output = ()> + 'static {
        let spv = self.spv;
        let stream = {
            self.reciver.for_each(move |(who, value)| {
                spv.submit_fc_transfer_tss(TxMessage::new(TxType::FCDeposit(
                    who,
                    TokenType::FC,
                    value,
                )));
                futures::future::ready(())
            })
        };
        stream
    }
}

type FcPubkeySender = mpsc::UnboundedReceiver<Vec<u8>>;

pub fn start_fc_service<A, B, C, Block>(
    client: Arc<C>,
    pool: Arc<A>,
    mut reciver: FcPubkeySender,
) -> impl Future<Output = ()> + 'static
where
    A: TransactionPool<Block = Block> + 'static,
    Block: BlockT,
    B: backend::Backend<Block> + Send + Sync + 'static,
    C: BlockBuilderProvider<B, Block, C>
        + HeaderBackend<Block>
        + ProvideRuntimeApi<Block>
        + BlockchainEvents<Block>
        + CallApiAt<Block>
        + Send
        + Sync
        + 'static,
    C::Api: VendorApi<Block>,
    Block::Hash: Into<sp_core::H256>,
{
    let key = sr25519::Pair::from_string(&format!("//{}", "Dave"), None)
        .expect("static values are valid; qed");

    let info = client.info();
    let at = BlockId::Hash(info.best_hash);

    let tx_sender = TxSender::new(
        client,
        pool,
        key,
        Arc::new(Mutex::new(PacketNonce {
            nonce: 0,
            last_block: at,
        })),
    );
    let (a, b) = unbundchannel();

    let fc_sender = FCSender::new(tx_sender, b);

    // loop to fetch Message from FileCoin & send to fc_sender
    main_loop(a, reciver);
    // main thread to revice & parse FileCoin Message and submit to filecoin
    fc_sender.sign_tss_message_start()
}

type Value = u128;
type MessageStreamR<V> = mpsc::UnboundedReceiver<(Vec<u8>, V)>;
type MessageStreamS<V> = mpsc::UnboundedSender<(Vec<u8>, V)>;
type CidBytes = Vec<u8>;

fn extract_message(message: UnsignedMessage) -> (CidBytes, originAddress, Vec<u8>, u128) {
    let revice_address = message.to.clone();

    let from_address = message.from.clone();
    let deposit_boolid = message.params.clone().into_inner();
    let deposit_amount = message.value.clone().to_u128().unwrap();

    let cid = message.cid().to_bytes();
    (cid, revice_address, deposit_boolid, deposit_amount)
}

pub fn unbundchannel() -> (MessageStreamS<Value>, MessageStreamR<Value>) {
    let (sender, reciver) = mpsc::unbounded::<(Vec<u8>, Value)>();
    (sender, reciver)
}

//const Address = Address::new_secp256k1_addr();

pub fn main_loop(sender: mpsc::UnboundedSender<(Vec<u8>, Value)>, mut reciver: FcPubkeySender) {
    thread::spawn(move || {
        let height = 0u64;

        let mut recv_addr: Vec<u8> = Vec::new();
        loop {
            thread::sleep(time::Duration::new(30, 0));
            match reciver.try_next() {
                Ok(Some(x)) => {
                    recv_addr = x;
                    break;
                }
                Ok(None) => {}
                Err(x) => {}
            }
        }
        println!("get recvice address {:?}", recv_addr);
        loop {
            thread::sleep(time::Duration::new(7, 0));
            let mut rt = Runtime::new().unwrap();
            let http = Http::new("http://47.52.21.141:1234/rpc/v0");
            let ret: TipSet = rt.block_on(http.chain_head()).unwrap();

            let new_height = ret.height as u64;
            if new_height == height {
                continue;
            }

            let mut message_set = HashMap::<Vec<u8>, (Vec<u8>, u128)>::new();
            let cids = ret.cids.clone();
            for cid in cids {
                println!("cids = {:?}", cid);
                let block_messages: BlockMessages =
                    rt.block_on(http.chain_get_block_messages(&cid)).unwrap();
                println!("block_messages = {:?}", block_messages);
                let signed_messages = block_messages.secpk_messages.clone();
                for message in signed_messages {
                    let (cid, revice_addr, who, val) = extract_message(message.message);
                    if revice_addr == originAddress::new_secp256k1_addr(&recv_addr).unwrap() {
                        message_set.insert(cid, (who, val));
                    }
                }
            }
            for (cid, (who, val)) in message_set {
                sender.unbounded_send((who, val));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        //cargo test --color=always --package fc-signer --lib tests::test -- --exact --nocapture
        let mut rt = Runtime::new().unwrap();
        let http = Http::new("http://47.52.21.141:1234/rpc/v0");
        let ret: TipSet = rt.block_on(http.chain_head()).unwrap();
        let cids = ret.cids.clone();
        let ret = rt
            .block_on(http.chain_get_block_messages(&cids[0]))
            .unwrap();

        for cid in cids {
            println!("cids = {:?}", cid);
            let bm: BlockMessages = rt.block_on(http.chain_get_block_messages(&cid)).unwrap();
            println!("block_messages = {:?}", bm);
            let signed_messages = bm.secpk_messages.clone();
            for message in signed_messages {
                //let (who,val) = extract_message(message.message);
            }
        }
    }
}
