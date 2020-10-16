use serde::{Serialize, Deserialize};

/// Unpadded size of a piece, in bytes
#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UnpaddedPieceSize(pub(crate) u64);