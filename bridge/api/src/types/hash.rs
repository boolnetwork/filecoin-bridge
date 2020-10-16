use fixed_hash::construct_fixed_hash;
use serde::{de, ser};
use super::utils::bytes_json;

construct_fixed_hash! {
    /// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
    pub struct H256(32);
}

pub type Randomness = H256;

// Implement JSON serialization for H256.
impl ser::Serialize for H256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        bytes_json::serialize(self.as_bytes(), serializer)
    }
}

// Implement JSON deserialization for H256.
impl<'de> de::Deserialize<'de> for H256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let bytes = bytes_json::deserialize(deserializer)?;
        if bytes.len() == H256::len_bytes() {
            Ok(H256::from_slice(bytes.as_slice()))
        } else {
            Err(de::Error::custom("H256 length must be 32 Bytes"))
        }
    }
}