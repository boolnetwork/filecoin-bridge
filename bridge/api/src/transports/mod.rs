use log::{debug};
use jsonrpc_core::{Response, Output, Request, Params, Call};
use serde::de::DeserializeOwned;
use crate::error::Result;

mod http;

pub use self::http::*;

/// Assigned RequestId
pub type RequestId = usize;

#[async_trait::async_trait]
pub trait Transport {
    /// Prepare serializable RPC call for given method with parameters.
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> (RequestId, Call);

    /// Execute prepared RPC call.
    async fn execute(&self, id: RequestId, request: &Request) -> Result<Response>;

    /// Send remote method with given parameters.
    async fn send<M, T>(&self, method: M, params: Params) -> Result<T>
        where
            M: Into<String> + Send,
            T: DeserializeOwned,
    {
        let (id, call) = self.prepare(method, params);
        let request = Request::Single(call);
        debug!(
            "Request: {}",
            serde_json::to_string(&request).expect("Serialize `Request` never fails")
        );

        let response = self.execute(id, &request).await?;
        debug!(
            "Response: {}",
            serde_json::to_string(&response).expect("Serialize `Response` never fails")
        );
        match response {
            Response::Single(Output::Success(success)) => {
                Ok(serde_json::from_value(success.result)?)
            }
            Response::Single(Output::Failure(failure)) => Err(failure.error.into()),
            Response::Batch(_) => panic!("Expected single, got batch"),
        }
    }
}