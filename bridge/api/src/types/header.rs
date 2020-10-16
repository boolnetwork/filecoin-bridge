use serde::{Deserialize, Serialize};
use cid::Cid;
use num_bigint::BigInt;

use super::address::Address;
use super::ticket::Ticket;
use super::proofs::{ElectionProof, PoStProof};
use super::crypto::Signature;
use super::utils::{vec_cid_json, cid_json, bytes_json, bigint_json};
use super::tipset::TipSet;

pub type ChainEpoch = i64;

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BeaconEntry {
    ///
    pub round: u64,
    ///
    #[serde(with = "bytes_json")]
    pub data: Vec<u8>,
}

/// The header part of the block.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockHeader {
    ///
    pub miner: Address,
    ///
    pub ticket: Ticket,
    ///
    pub election_proof: ElectionProof,
    ///
    pub beacon_entries: Option<Vec<BeaconEntry>>,
    ///
    #[serde(rename = "WinPoStProof")]
    pub win_post_proof: Vec<PoStProof>,
    ///
    #[serde(with = "vec_cid_json")]
    pub parents: Vec<Cid>,
    ///
    #[serde(with = "bigint_json")]
    pub parent_weight: BigInt,
    ///
    pub height: ChainEpoch,
    ///
    #[serde(with = "cid_json")]
    pub parent_state_root: Cid,
    ///
    #[serde(with = "cid_json")]
    pub parent_message_receipts: Cid,
    ///
    #[serde(with = "cid_json")]
    pub messages: Cid,
    ///
    #[serde(rename = "BLSAggregate")]
    pub bls_aggregate: Signature,
    ///
    pub timestamp: u64,
    ///
    pub block_sig: Signature,
    ///
    pub fork_signaling: u64,
    /*
    /// internal
    #[serde(skip)]
    validated: bool, // true if the signature has been validated
    */
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeadChangeType {
    Revert,
    Apply,
    Current,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HeadChange {
    pub r#type: HeadChangeType,
    pub val: TipSet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_header_json() {
        let json = r#"{"Miner":"t01000","Ticket":{"VRFProof":"k4aywRis+mYWN56o3OQOAxEFxKSp777TR1h8hcTEeWlLwvERi2oXnTE7xzS0uoLICnEhoGs9BL5MGDYpf3dfmvLD+h7iBimSpl6rY7bysDbuKreKXa9GwAPN3fQqJB1O"},"ElectionProof":{"VRFProof":"g7Ki1qDQtj0Q1o2bRpHZqD++UqFfjPaOJ5WYT2wJjCgGxg/+2L4cozSU/F7IzGIfE1E79C0brMGROGCLMui4qiSZr1D9sJmn+EBwrLjbqpiJEVXqoXoFEkw7/xpFjIat"},"BeaconEntries":null,"WinPoStProof":[{"RegisteredProof":9,"ProofBytes":"scKG734ZjZjlLv1I9z/7R4qmL3M0kpkTKtBa00pGVxA8cd3myhwhocX8BL4pHl8QmMbkPqp5iXh0sbCdJjbJ6/OmAvpATiAYf3R7pTMOdkLvxFofq4NDEtv8t/I4fnOJAcTvG0ozeNA3MM0KjR2X+kfz4Fo4kVflCdhcT9cKlYBO7IiVKYm/RN0zyvJi6pzhmBtryhGzYyNYv3jWVde8qUtIQnD0169SzYVrbZlfF4ydpgGj5PriYRXrCTi9DXmz"}],"Parents":[{"/":"bafy2bzacebwut2il7udv5d3yzscpwbomvj5ocq6lkxh4kcusiy5juesvpun4c"}],"ParentWeight":"629642112","Height":149063,"ParentStateRoot":{"/":"bafy2bzaceae7pqh2wupmp3fqnlbsxx2czjku5rbisl3qdtaa5mehs2hkjak3a"},"ParentMessageReceipts":{"/":"bafy2bzaceaa43et73tgxsoh2xizd4mxhbrcfig4kqp25zfa5scdgkzppllyuu"},"Messages":{"/":"bafy2bzacecgw6dqj4bctnbnyqfujltkwu7xc7ttaaato4i5miroxr4bayhfea"},"BLSAggregate":{"Type":2,"Data":"wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"Timestamp":1592693392,"BlockSig":{"Type":2,"Data":"l+3ZTa9Q1mj8UcVMAetZSuZphQQJUDfaSbXbZf6rNTBhrqE7feLMcTCCMcUOClNnFH+P8HQmOZ8YwH47vU2vw6maLU33bS5Bc6+MvF7gjFx2pRHgq5GM8SPunDA3fKFe"},"ForkSignaling":0}"#;

        let block_header = serde_json::from_str::<BlockHeader>(json);
        println!("{:?}", block_header);
    }
}