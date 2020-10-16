use serde::{de, ser};
use super::utils::bytes_json;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug, Hash, Default)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_inner(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_mut_inner(&mut self) -> &mut[u8] {
        self.0.as_mut_slice()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.as_inner()
    }
}

impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_inner()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Implement JSON serialization of Vec<u8> using base64.
impl ser::Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        bytes_json::serialize(self.as_inner(), serializer)
    }
}

/// Implement JSON deserialization of Vec<u8> using base64.
impl<'de> de::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Bytes, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        Ok(Self(bytes_json::deserialize(deserializer)?))
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug, Hash, Default)]
pub struct BytesRef<'a>(&'a [u8]);

impl<'a> BytesRef<'a> {
    /// Don't consume the wrapper, borrowing the underlying &[u8].
    pub fn as_inner(&self) -> &[u8] {
        self.0
    }
}

impl<'a> AsRef<[u8]> for BytesRef<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_inner()
    }
}

impl<'a> From<&'a [u8]> for BytesRef<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

/// Implement JSON serialization of &[u8] using base64.
impl<'a> ser::Serialize for BytesRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        bytes_json::serialize(self.as_inner(), serializer)
    }
}
