use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use super::message::SignedMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MpoolUpdate {
    pub r#type: MpoolChange,
    pub message: SignedMessage,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum MpoolChange {
    MpoolAdd = 0,
    MpoolRemove = 1,
}
