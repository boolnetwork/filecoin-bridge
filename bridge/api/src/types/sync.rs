use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use super::header::ChainEpoch;
use super::tipset::TipSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SyncState {
    pub active_syncs: Vec<ActiveSync>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActiveSync {
    pub base: TipSet,
    pub target: TipSet,

    pub stage: SyncStateStage,
    pub height: ChainEpoch,

    pub start: String,
    pub end: String,
    pub message: String,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum SyncStateStage {
    StageIdle = 0,
    StageHeaders = 1,
    StagePersistHeaders = 2,
    StageMessages = 3,
    StageSyncComplete = 4,
    StageSyncErrored = 5,
}
