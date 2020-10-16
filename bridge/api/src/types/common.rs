use std::fmt;
use libp2p_core::{Multiaddr, PeerId};
use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use super::utils::peerid_json;

/// The permission of API.
#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Read-only permission
    Read,
    /// Write permission
    Write,
    /// Use wallet keys for signing
    Sign,
    /// Manage permissions
    Admin,
}

impl Default for Permission {
    fn default() -> Self {
        Permission::Read
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => f.write_str("read"),
            Permission::Write => f.write_str("write"),
            Permission::Sign => f.write_str("sign"),
            Permission::Admin => f.write_str("admin"),
        }
    }
}


/// Connectedness signals the capacity for a connection with a given node.
/// It is used to signal to services and other peers whether a node is reachable.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
pub enum Connectedness {
    /// NotConnected means no connection to peer, and no extra information (default)
    NotConnected = 0,
    /// Connected means has an open, live connection to peer
    Connected = 1,
    /// CanConnect means recently connected to peer, terminated gracefully
    CanConnect = 2,
    /// CannotConnect means recently attempted connecting but failed to connect.
    /// (should signal "made effort, failed")
    CannotConnect = 3,
}

/// AddrInfo is a small struct used to pass around a peer with a set of addresses.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeerAddrInfo {
    /// peer ID.
    #[serde(rename = "ID")]
    #[serde(with = "peerid_json")]
    pub id: PeerId,
    /// A set of addresses.
    pub addrs: Vec<Multiaddr>,
}