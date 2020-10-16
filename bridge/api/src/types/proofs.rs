use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use super::utils::bytes_json;

#[repr(u64)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Serialize_repr, Deserialize_repr)]
pub enum RegisteredProof {
    StackedDRG32GiBSeal = 1,
    // StackedDRG32GiBPoSt = 2, // No longer used
    StackedDRG2KiBSeal = 3,
    // StackedDRG2KiBPoSt = 4, // No longer used
    StackedDRG8MiBSeal = 5,
    // StackedDRG8MiBPoSt = 6, // No longer used
    StackedDRG512MiBSeal = 7,
    // StackedDRG512MiBPoSt = 8, // No longer used
    StackedDRG2KiBWinningPoSt = 9,
    StackedDRG2KiBWindowPoSt = 10,
    StackedDRG8MiBWinningPoSt = 11,
    StackedDRG8MiBWindowPoSt = 12,
    StackedDRG512MiBWinningPoSt = 13,
    StackedDRG512MiBWindowPoSt = 14,
    StackedDRG32GiBWinningPoSt = 15,
    StackedDRG32GiBWindowPoSt = 16,
    StackedDRG64GiBSeal = 17,
    StackedDRG64GiBWinningPoSt = 18,
    StackedDRG64GiBWindowPoSt = 19,
}

/// The PoSt election proof of space/time
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct ElectionProof {
    /// VRF proof
    #[serde(rename = "VRFProof")]
    #[serde(with = "bytes_json")]
    pub vrf_proof: Vec<u8>,
}

/// The PoSt proof.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoStProof {
    pub registered_proof: RegisteredProof,
    #[serde(with = "bytes_json")]
    pub proof_bytes: Vec<u8>,
}