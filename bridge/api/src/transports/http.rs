use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use jsonrpc_core::{Request, Response, Params, Call, Version, MethodCall, Id};
use crate::error::Result;
use crate::transports::{Transport, RequestId};

#[derive(Clone)]
pub struct Http {
    id: Arc<AtomicUsize>,
    url: String,
    bearer_auth: Option<String>,
    client: reqwest::Client,
}

impl Http {
    pub fn new(url: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("ClientBuilder config is valid; qed");

        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth: None,
            client,
        }
    }

    async fn send_request(&self, request: &Request) -> Result<Response> {
        let builder = self.client.post(&self.url).json(request);
        let builder = if let Some(token) = &self.bearer_auth {
            builder.bearer_auth(token)
        } else {
            builder
        };
        Ok(builder.send().await?.json().await?)
    }
}

#[async_trait::async_trait]
impl Transport for Http {
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> (RequestId, Call) {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        let call = Call::MethodCall(MethodCall {
            jsonrpc: Some(Version::V2),
            id: Id::Num(id as u64),
            method: method.into(),
            params,
        });
        (id, call)
    }

    async fn execute(&self, _id: RequestId, request: &Request) -> Result<Response> {
        self.send_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::Value;
    use tokio::runtime::Runtime;

    #[test]
    fn basic_test() {
        let mut rt = Runtime::new().unwrap();
        let http = Http::new("http://127.0.0.1:1234/rpc/v0");
        // Filecoin.Version need read permission
        let version: Value = rt.block_on(http.send("Filecoin.Version", Params::Array(vec![]))).unwrap();

        println!("Version: {:?}", version);
    }
}