use serde::Serialize;
use serde_json::{value::Serializer, Value};

#[inline]
pub fn serialize<T: Serialize>(value: &T) -> Value {
    value
        .serialize(Serializer)
        .expect("Types never fail to serialize")
}
