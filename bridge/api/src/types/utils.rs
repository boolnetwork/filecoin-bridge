use serde::{de::{self, SeqAccess, Visitor}};
use serde::Deserialize;
use std::marker::PhantomData;
use std::fmt;

pub mod bigint_json {
    use num_bigint::BigInt;
    use serde::{de, ser, Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct BigIntWrapper(#[serde(with = "self")] pub BigInt);

    impl BigIntWrapper {
        pub fn into_inner(self) -> BigInt {
            self.0
        }
    }

    /// JSON serialization
    pub fn serialize<S>(int: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        int.to_string().serialize(serializer)
    }

    /// JSON deserialization
    pub fn deserialize<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let bigint = String::deserialize(deserializer)?
            .parse::<BigInt>()
            .map_err(|e| de::Error::custom(e.to_string()))?;
        Ok(bigint)
    }
}

pub mod bytes_json {
    use serde::{de, ser, Deserialize, Serialize};
    /// Implement JSON serialization of Vec<u8> using base64.
    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        base64::encode(bytes).serialize(serializer)
    }

    /// Implement JSON deserialization of Vec<u8> using base64.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        base64::decode(&String::deserialize(deserializer)?)
            .map_err(|err| de::Error::custom(format!("base64 decode error: {}", err)))
    }
}

pub mod cid_json {
    use cid::Cid;
    use serde::{de, ser, Deserialize, Serialize};

    /// Wrapper for serializing and deserializing a Cid from JSON.
    #[derive(Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct CidJson(#[serde(with = "self")] pub Cid);

    /// Wrapper for serializing a cid reference to JSON.
    #[derive(Serialize)]
    #[serde(transparent)]
    pub struct CidJsonRef<'a>(#[serde(with = "self")] pub &'a Cid);

    impl From<CidJson> for Cid {
        fn from(wrapper: CidJson) -> Self {
            wrapper.0
        }
    }

    pub fn serialize<S>(c: &Cid, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        CidMap { cid: c.to_string() }.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Cid, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let CidMap { cid } = Deserialize::deserialize(deserializer)?;
        cid.parse().map_err(de::Error::custom)
    }

    /// Struct just used as a helper to serialize a cid into a map with key "/"
    #[derive(Serialize, Deserialize)]
    struct CidMap {
        #[serde(rename = "/")]
        cid: String,
    }
}

#[derive(Default)]
pub struct GoVecVisitor<T, D = T> {
    return_type: PhantomData<T>,
    deserialize_type: PhantomData<D>,
}

impl<T, D> GoVecVisitor<T, D> {
    pub fn new() -> Self {
        Self {
            return_type: PhantomData,
            deserialize_type: PhantomData,
        }
    }
}

impl<'de, T, D> Visitor<'de> for GoVecVisitor<T, D>
    where
        T: From<D>,
        D: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a vector of serializable objects or null")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
        where
            A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element::<D>()? {
            vec.push(T::from(elem));
        }
        Ok(vec)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Ok(Vec::new())
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.visit_none()
    }
}

pub mod vec_cid_json {
    use cid::Cid;
    use serde::{de, ser};
    use super::cid_json::*;
    use super::GoVecVisitor;
    use serde::ser::SerializeSeq;

    pub fn serialize<S>(m: &[Cid], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(m.len()))?;
        for e in m {
            seq.serialize_element(&CidJsonRef(e))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Cid>, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(GoVecVisitor::<Cid, CidJson>::new())
    }
}

pub mod peerid_json {
    use libp2p_core::PeerId;
    use serde::{de, ser, Deserialize, Serialize};

    /// A wrapper of `libp2p_core::PeerId` that implement CBOR and JSON serialization/deserialization.
    #[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct PeerIdWrapper(#[serde(with = "self")] pub libp2p_core::PeerId);

    impl PeerIdWrapper {
        /// Consumes the wrapper, returning the underlying libp2p_core::PeerId.
        pub fn into_inner(self) -> libp2p_core::PeerId {
            self.0
        }

        /// Don't consume the wrapper, borrowing the underlying libp2p_core::PeerId.
        pub fn as_inner(&self) -> &libp2p_core::PeerId {
            &self.0
        }

        /// Don't consume the wrapper, mutable borrowing the underlying libp2p_core::PeerId.
        pub fn as_mut_inner(&mut self) -> &mut libp2p_core::PeerId {
            &mut self.0
        }
    }

    impl From<libp2p_core::PeerId> for PeerIdWrapper {
        fn from(peer_id: libp2p_core::PeerId) -> Self {
            Self(peer_id)
        }
    }

    #[derive(Serialize)]
    #[serde(transparent)]
    pub struct PeerIdRefWrapper<'a>(#[serde(with = "self")] pub &'a libp2p_core::PeerId);

    impl<'a> PeerIdRefWrapper<'a> {
        /// Don't consume the wrapper, borrowing the underlying libp2p_core::PeerId.
        pub fn as_inner(&self) -> &libp2p_core::PeerId {
            self.0
        }
    }

    impl<'a> From<&'a libp2p_core::PeerId> for PeerIdRefWrapper<'a> {
        fn from(peer_id: &'a libp2p_core::PeerId) -> Self {
            Self(peer_id)
        }
    }

    /// JSON serialization
    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        peer_id.to_string().serialize(serializer)
    }

    /// JSON deserialization
    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let peer_id = String::deserialize(deserializer)?
            .parse::<libp2p_core::PeerId>()
            .map_err(|err| de::Error::custom(err.to_string()))?;
        Ok(peer_id)
    }
}
