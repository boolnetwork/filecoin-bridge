//! lotus api Error
use jsonrpc_core::Error as RPCError;
use derive_more::{Display, From};
use serde_json::Error as SerdeError;
use reqwest::Error as ReqwestError;
use std::io::Error as IoError;
use serde::export::From;

/// Lotus `Result` type.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Errors which can occur when attempting to generate resource uri.
#[derive(Debug, Display, From)]
pub enum Error {
    /// server is unreachable
    #[display(fmt = "Server is unreachable")]
    Unreachable,
    /// json error
    #[display(fmt = "Json error: {}", _0)]
    #[from(ignore)]
    Json(String),
    /// transport error
    #[display(fmt = "Transport error: {}", _0)]
    #[from(ignore)]
    Transport(String),
    /// rpc error
    #[display(fmt = "RPC error: {:?}", _0)]
    Rpc(RPCError),
    /// io error
    #[display(fmt = "IO error: {}", _0)]
    Io(IoError),
    /// signing error
    #[display(fmt = "Signing error: {}", _0)]
    Signing(String),
    /// Lotus internal error
    #[display(fmt = "Internal lotus error")]
    Internal,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;
        match *self {
            Unreachable | Signing(_) | Transport(_) | Json(_) | Internal => None,
            Rpc(ref e) => Some(e),
            Io(ref e) => Some(e),
        }
    }
}

impl From<SerdeError> for Error {
    fn from(e: SerdeError) -> Self {
        Error::Json(format!("{:?}", e))
    }
}

impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Self {
        Error::Transport(format!("{:?}", e))
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        use self::Error::*;
        match self {
            Unreachable => Unreachable,
            Json(e) => Json(e.clone()),
            Transport(s) => Transport(s.clone()),
            Rpc(e) => Rpc(e.clone()),
            Io(e) => Io(IoError::from(e.kind())),
            Signing(e) => Signing(e.clone()),
            Internal => Internal,
        }
    }
}

#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use self::Error::*;
        match (self, other) {
            (Unreachable, Unreachable) | (Internal, Internal) => true,
            (Json(a), Json(b)) | (Transport(a), Transport(b)) => {
                a == b
            }
            (Rpc(a), Rpc(b)) => a == b,
            (Io(a), Io(b)) => a.kind() == b.kind(),
            (Signing(a), Signing(b)) => a == b,
            _ => false,
        }
    }
}
