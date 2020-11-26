#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use parking_lot::Mutex;
use std::{sync::Arc, thread, time, marker::PhantomData, collections::HashMap};
use futures::{channel::mpsc, prelude::*};
use tokio::runtime::Runtime;

use num_traits::cast::ToPrimitive;
use bridge::{PacketNonce, SuperviseClient, TokenType, TxMessage, TxSender, TxType, ChainState};
use sp_transaction_pool::{TransactionPool};
use filecoin_bridge_runtime::apis::VendorApi;
use sc_block_builder::BlockBuilderProvider;
use sc_client_api::{backend, BlockchainEvents};
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::{sr25519, Pair};
use sp_runtime::{generic::{BlockId}, traits::{Block as BlockT}};

use lotus_api_forest::{self, Http as filecoin_http, api::ChainApi};
use interpreter::{self, BlockMessages};
use forest_blocks::{self, Tipset, tipset::tipset_json::TipsetJson};
use forest_message::{self, UnsignedMessage};
use forest_address::{self, Address};
use forest_encoding::Cbor;
use rpc::BlockMessages as forest_BlockMessages;
use serde_json::*;
use serde::*;

use std::time::Duration;
lazy_static! {
    pub static ref STORE_LIST: Mutex<Vec<&'static str>> = Mutex::new(vec!["test"]);
}

type FCValue = u128;
type FCMessageCidBytes = Vec<u8>;
type SubTargetAccountId = Vec<u8>;
type FCFromAddress = Vec<u8>;

type StreamData<V> = (SubTargetAccountId, V, FCFromAddress);
type MessageStreamR<V> = mpsc::UnboundedReceiver<StreamData<V>>;
type MessageStreamS<V> = mpsc::UnboundedSender<StreamData<V>>;
type DepositData<V> = (SubTargetAccountId, V, FCFromAddress);
type ExtractMessage<V> = (FCMessageCidBytes, Address, SubTargetAccountId, V, FCFromAddress);

fn extract_message(message: UnsignedMessage) -> ExtractMessage<FCValue> {

    let revice_address = message.to.clone();
    let from_address = message.from.to_bytes();
    let deposit_boolid = message.params.bytes().to_vec();
    let deposit_amount = message.value.clone().to_u128().unwrap();
    let cid = message.cid().unwrap().to_bytes();
    println!("extract_message deposit_boolid={:?} deposit_amount={:?}",deposit_boolid,deposit_amount);

    (cid, revice_address, deposit_boolid, deposit_amount, from_address)
}

pub fn get_fc_message_parse_channel() -> (MessageStreamS<FCValue>, MessageStreamR<FCValue>) {
    let (sender, reciver) = mpsc::unbounded::<(Vec<u8>, FCValue, Vec<u8>)>();
    (sender, reciver)
}

#[derive(Debug)]
pub struct FCMessageForward<V, B> {
    pub spv: Arc<V>,
    pub reciver: MessageStreamR<FCValue>,
    pub a: std::marker::PhantomData<B>,
}

impl<V, B> FCMessageForward<V, B>
where
    V: SuperviseClient<B> + Send + Sync + 'static,
    B: BlockT,
{
    pub fn new(spv: Arc<V>, rec: MessageStreamR<FCValue>) -> Self {
        FCMessageForward {
            spv: spv,
            reciver: rec,
            a: PhantomData,
        }
    }

    fn submit_tx(&self, data: TxMessage) {
        self.spv.submit_fc_transfer_tss(data);
    }

    fn start_sign_push_fc_message(self) -> impl Future<Output = ()> + 'static {
        let spv = self.spv;
        let stream = {
            self.reciver.for_each(move |(who, value, from)| {
                spv.submit_fc_transfer_tss(TxMessage::new(TxType::FCDeposit(
                    who,
                    TokenType::FC,
                    value,
                    from,
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
        client.clone(),
        pool,
        key,
        Arc::new(Mutex::new(PacketNonce {
            nonce: 0,
            last_block: at,
        })),
    );

    let tx_sender_arc = Arc::new(tx_sender);

    let (fc_parse_sender,
        fc_parse_recvier) = get_fc_message_parse_channel();

    let fc_message_forward =
        FCMessageForward::new(tx_sender_arc, fc_parse_recvier);

    // to fetch Message from FileCoin & send to fc_sender
    fc_message_fetch_parse(fc_parse_sender, reciver,ChainState::new(client));
    // to revice & parse FileCoin Message and submit to filecoin
    fc_message_forward.start_sign_push_fc_message()
}

pub fn fc_message_fetch_parse<Block,B,C>(sender: MessageStreamS<FCValue>, _reciver: FcPubkeySender, state: ChainState<Block,B,C>)
    where
        Block: BlockT,
        B: backend::Backend<Block> + Send + Sync + 'static,
        C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
        + CallApiAt<Block> + Send + Sync + 'static,
        C::Api: VendorApi<Block>,
{
    thread::spawn(move || {
        let mut height = 0u64;

        let mut recv_addr: Vec<u8> = Vec::new();
        loop {
            thread::sleep(time::Duration::new(10, 0));
            let pubkey = state.tss_pubkey();
            if pubkey.len() == 0 || pubkey.len() != 65{
                continue;
            }else{
                recv_addr = pubkey;
                break;
            }
        }
        let addr = Address::new_secp256k1(&recv_addr).unwrap();
        println!("token recvice address in Filecoin is {}",addr );

        loop {
            thread::sleep(time::Duration::new(1, 0));
            let mut rt = Runtime::new().unwrap();
            let http = filecoin_http::new("http://127.0.0.1:1234/rpc/v0");
            let ret_json: TipsetJson = rt.block_on(http.chain_head()).unwrap();
            let ret: Tipset = ret_json.into();

            let new_height = ret.epoch() as u64;
            if new_height == height {
                continue;
            }
            height = new_height;
            let mut message_set = HashMap::<Vec<u8>, DepositData<FCValue>>::new();
            let cids = ret.cids();
            for cid in cids {
                println!("[filecoin block] cids = {:?} height={:?}", cid, height);
                let block_messages: forest_BlockMessages =
                    rt.block_on(http.chain_get_block_messages(&cid)).unwrap();
                let signed_bls_messages = block_messages.bls_msg.clone();
                for message in signed_bls_messages {
                    let (cid, revice_addr, who, val, from) = extract_message(message.clone());
                    if revice_addr == Address::new_secp256k1(&recv_addr).unwrap() {
                        message_set.insert(cid, (who, val, from));
                    }
                }
            }
            for (_cid, (who, val, from)) in message_set {
                sender.unbounded_send((who, val, from));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpc::BlockMessages as forest_BlockMessages;
    use lotus_api_forest::api::MpoolApi;
    #[test]
    fn test() {
        //cargo test --color=always --package fc-signer --lib tests::test -- --exact --nocapture
        let mut rt = Runtime::new().unwrap();
        let http = lotus_api_forest::Http::new("http://127.0.0.1:1234/rpc/v0");
        let ret2: forest_blocks::tipset::tipset_json::TipsetJson = rt.block_on(http.chain_head()).unwrap();
        let ret: Tipset = ret2.into();
        let cids = ret.cids().clone();
        println!("cids = {:?}", cids);
        let block_messages: forest_BlockMessages =
            rt.block_on(http.chain_get_block_messages(&cids[0])).unwrap();
        // let block_messages: BlockMessages = block_messages_rpc.into();
        let signed_messages = block_messages.secp_msg.clone();
        println!("signed_messages = {:?}", signed_messages);

        let addr= forest_address::Address::new_id(1);
        let nonce:u64 = rt.block_on(http.mpool_get_nonce(&addr)).unwrap();
        println!("nonce = {:?}", nonce);

//        let message = forest_message::UnsignedMessage::
//        let res = rt.block_on(http.mpool_push(&message)).unwrap();
    }
}
