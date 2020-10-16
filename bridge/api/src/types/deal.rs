use serde::{Serialize, Deserialize};
use num_bigint::BigInt;
use cid::Cid;
use libp2p_core::PeerId;
use super::utils::{bigint_json, peerid_json, cid_json};
use super::address::Address;
use super::header::ChainEpoch;
use super::piece::UnpaddedPieceSize;

pub type DealId = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileRef {
    pub path: String,
    #[serde(rename = "IsCAR")]
    pub is_car: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartDealParams {
    // pub data: storagemarket::DataRef,
    pub wallet: Address,
    pub miner: Address,
    #[serde(with = "bigint_json")]
    pub epoch_price: BigInt,
    pub min_blocks_duration: u64,
    pub deal_start_epoch: ChainEpoch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Import {
    // pub status: filestore::Status,
    #[serde(with = "cid_json")]
    pub key: Cid,
    pub file_path: String,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DealInfo {
    #[serde(with = "cid_json")]
    pub proposal_cid: Cid,
    // pub state: storagemarket::StorageDealStatus,
    pub message: String,    // more information about deal state, particularly errors
    pub provider: Address,
    #[serde(rename = "PieceCID")]
    #[serde(with = "cid_json")]
    pub piece_cid: Cid,
    pub size: u64,
    #[serde(with = "bigint_json")]
    pub price_per_epoch: BigInt,
    pub duration: u64,
    #[serde(rename = "DealID")]
    pub deal_id: DealId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryOffer {
    pub err: String,
    #[serde(with = "cid_json")]
    pub root: Cid,
    pub size: u64,
    #[serde(with = "bigint_json")]
    pub min_price: BigInt,
    pub payment_interval: u64,
    pub payment_interval_increase: u64,
    pub miner: Address,
    #[serde(rename = "MinerPeerID")]
    #[serde(with = "peerid_json")]
    pub miner_peer_id: PeerId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RetrievalOrder {
    // TODO: make this less unixfs specific
    #[serde(with = "cid_json")]
    pub root: Cid,
    pub size: u64,
    // TODO: support offset
    #[serde(with = "bigint_json")]
    pub total: BigInt,
    pub payment_interval: u64,
    pub payment_interval_increase: u64,
    pub client: Address,
    pub miner: Address,
    #[serde(rename = "MinerPeerID")]
    #[serde(with = "peerid_json")]
    pub miner_peer_id: PeerId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommPRet {
    #[serde(with = "cid_json")]
    pub root: Cid,
    pub size: UnpaddedPieceSize,
}