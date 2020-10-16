use serde::{Serialize, Deserialize};
use num_bigint::BigInt;
use cid::Cid;
use super::utils::cid_json;
use super::proofs::RegisteredProof;

/// SectorNumber is a numeric identifier for a sector. It is usually relative to a miner.
pub type SectorNumber = u64;

/// The unit of storage power (measured in bytes)
pub type StoragePower = BigInt;

/// The quality of sector.
pub type SectorQuality = BigInt;

/// The unit of spacetime committed to the network
pub type SpaceTime = BigInt;

/// SectorSize indicates one of a set of possible sizes in the network.
pub type SectorSize = u64;

/// Information about a sector necessary for PoSt verification.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SectorInfo {
    /// RegisteredProof used when sealing - needs to be mapped to PoSt registered proof when used to verify a PoSt
    pub registered_proof: RegisteredProof,
    pub sector_number: SectorNumber,
    #[serde(rename = "SealedCID")]
    #[serde(with = "cid_json")]
    pub sealed_cid: Cid,
}