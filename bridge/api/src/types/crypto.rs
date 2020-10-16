use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use std::convert::TryFrom;
use thiserror::Error;
use super::bytes::Bytes;
use super::address::Protocol;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Serialize_repr, Deserialize_repr)]
pub enum SignatureType {
    /// The `Secp256k1` signature.
    Secp256k1 = 1,
    /// The `BLS` signature.
    Bls = 2,
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum CryptoError {
    /// Unknown signature type.
    #[error("unknown signature type: {0}")]
    UnknownSignatureType(u8),
    /// Secp256k1 error.
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] secp256k1::Error),
    /// BLS error.
    #[error("bls error: {0}")]
    Bls(String),
    /// Signature and Address are not match
    #[error("signature and address is not same type, signature:{:0?}, addr:{1}")]
    NotSameType(SignatureType, Protocol),
    /// Signature verify failed
    #[error("signature verify failed")]
    VerifyFailed,
}

impl Default for SignatureType {
    fn default() -> Self {
        SignatureType::Bls
    }
}

impl TryFrom<u8> for SignatureType {
    type Error = CryptoError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SignatureType::Secp256k1),
            2 => Ok(SignatureType::Bls),
            _ => Err(CryptoError::UnknownSignatureType(value)),
        }
    }
}

impl From<SignatureType> for u8 {
    fn from(ty: SignatureType) -> Self {
        match ty {
            SignatureType::Secp256k1 => 1,
            SignatureType::Bls => 2,
        }
    }
}

/// The general signature structure.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Signature {
    /// The signature type.
    r#type: SignatureType,
    /// Tha actual signature bytes.
    data: Bytes,
}

impl Signature {
    pub fn new_secp(sig:Vec<u8>) -> Self{
        Self{
            r#type: SignatureType::Secp256k1,
            data: Bytes::from(sig)
        }
    }
}
#[repr(u8)]
#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum DomainSeparationTag {
    TicketProduction = 1,
    ElectionProofProduction,
    WinningPoStChallengeSeed,
    WindowedPoStChallengeSeed,
    SealRandomness,
    InteractiveSealChallengeSeed,
    WindowedPoStDeadlineAssignment,
}

#[cfg(test)]
mod tests {
    use super::Signature;
    use crate::types::crypto::{SignatureType, Bytes};
    use serde_json;

    #[test]
    fn signature_json_serde() {
        let s = Signature {
            r#type: SignatureType::Secp256k1,
            data: Bytes::default(),
        };

        println!("s {}", serde_json::to_string(&s).unwrap());
    }
}