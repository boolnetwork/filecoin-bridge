//mod common;
mod chain;
//mod wallet;
//mod sync;
mod mpool;
//mod miner;
//mod client;

//pub use common::CommpnApi;
pub use chain::ChainApi;
//pub use wallet::WalletApi;
//pub use sync::SyncApi;
pub use mpool::MpoolApi;
//pub use miner::MinerApi;
//pub use client::ClientApi;

use crate::transports::{Http, Transport};
use crate::error::Result;
use serde_json::Value;
use jsonrpc_core::Params;

#[async_trait::async_trait]
pub trait JsonApi {
    /// Send Rpc request.
    async fn request<M, T>(&self, method: M, params: Vec<Value>) -> Result<T>
        where
            M: AsRef<str> + Send,
            T: serde::de::DeserializeOwned;
}

#[async_trait::async_trait]
impl JsonApi for Http {
    async fn request<M, T>(&self, method: M, params: Vec<Value>) -> Result<T>
        where
            M: AsRef<str> + Send,
            T: serde::de::DeserializeOwned,
    {
        Ok(self
            .send(
                format!("Filecoin.{}", method.as_ref()),
                Params::Array(params),
            )
            .await?)
    }
}

//impl CommpnApi for Http {}
impl ChainApi for Http {}
//impl WalletApi for Http {}
//impl SyncApi for Http {}
impl MpoolApi for Http {}
//impl MinerApi for Http {}
