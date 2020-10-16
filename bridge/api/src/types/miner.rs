use serde::{Serialize, Deserialize};
use num_bigint::BigInt;
use super::address::Address;
use super::header::{BeaconEntry, ChainEpoch};
use super::ticket::Ticket;
use super::proofs::{ElectionProof, PoStProof};
use super::message::SignedMessage;
use super::tipset::TipSetKey;
use super::sector::{SectorSize, SectorInfo};
use super::utils::bigint_json;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MiningBaseInfo {
    #[serde(with = "bigint_json")]
    pub miner_power: BigInt,
    #[serde(with = "bigint_json")]
    pub network_power: BigInt,
    pub sectors: Vec<SectorInfo>,
    pub worker_key: Address,
    pub sector_size: SectorSize,
    pub prev_beacon_entry: BeaconEntry,
    pub beacon_entries: Vec<BeaconEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockTemplate {
    pub miner: Address,
    pub parents: TipSetKey,
    pub ticket: Ticket,
    pub eproof: ElectionProof,
    pub beacon_values: Vec<BeaconEntry>,
    pub messages: Vec<SignedMessage>,
    pub epoch: ChainEpoch,
    pub timestamp: u64,
    #[serde(rename = "WinningPoStProof")]
    pub winning_post_proof: Vec<PoStProof>,
}
