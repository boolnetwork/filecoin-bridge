use cid::Cid;
use serde::{Deserialize, Serialize};
use super::header::BlockHeader;
use super::message::{UnsignedMessage, SignedMessage};
use super::utils::vec_cid_json;

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Block {
    /// The block header.
    pub header: BlockHeader,
    /// The `BLS` messages.
    pub bls_messages: Vec<UnsignedMessage>,
    /// The `Secp256k1` messages.
    pub secpk_messages: Vec<SignedMessage>,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockMsg {
    /// The block header.
    pub header: BlockHeader,
    /// The CIDs of `BLS` messages.
    #[serde(with = "vec_cid_json")]
    pub bls_messages: Vec<Cid>,
    /// The CIDs of `Secp256k1` messages.
    #[serde(with = "vec_cid_json")]
    pub secpk_messages: Vec<Cid>,
}