use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{TipSetKey, SignedMessage, Cid, UnsignedMessage, Address, MpoolUpdate, BigInt,
BigIntWrapper, CidJson};
use forest_address;
use forest_message;
#[async_trait::async_trait]
pub trait MpoolApi: JsonApi {
    async fn mpool_pending(&self, key: &TipSetKey) -> Result<Vec<SignedMessage>> {
        self.request("MpoolPending", vec![helper::serialize(key)])
            .await
    }

    async fn mpool_push(&self, signed_msg: &forest_message::SignedMessage) -> Result<forest_cid::Cid> {
        let cid: forest_cid::json::CidJson = self.request("MpoolPush", vec![helper::serialize(signed_msg)])
            .await?;
        Ok(cid.into())
    }

    // get nonce, sign, push
    async fn mpool_push_message(&self, msg: &UnsignedMessage) -> Result<SignedMessage> {
        self.request("MpoolPushMessage", vec![helper::serialize(msg)])
            .await
    }

    async fn mpool_get_nonce(&self, addr: &forest_address::Address) -> Result<u64> {
        self.request("MpoolGetNonce", vec![helper::serialize(addr)])
            .await
    }

//    async fn mpool_sub(&self) -> Result<(SubscriptionId, NotificationStream<MpoolUpdate>)> {
//        self.subscribe("MpoolSub", vec![]).await
//    }

    async fn mpool_estimate_gas_price(
        &self,
        nblocksincl: u64,
        addr: &Address,
        gas_limit: i64,
        key: &TipSetKey,
    ) -> Result<BigInt> {
        let price: BigIntWrapper = self
            .request(
                "MpoolEstimateGasPrice",
                vec![
                    helper::serialize(&nblocksincl),
                    helper::serialize(addr),
                    helper::serialize(&gas_limit),
                    helper::serialize(key),
                ],
            )
            .await?;
        Ok(price.into_inner())
    }
}