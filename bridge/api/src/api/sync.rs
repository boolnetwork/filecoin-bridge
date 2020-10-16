use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{SyncState, Cid, CidJsonRef, BlockMsg};

#[async_trait::async_trait]
pub trait SyncApi: JsonApi {
    async fn sync_state(&self) -> Result<SyncState> {
        self.request("SyncState", vec![]).await
    }

    async fn sync_submit_block(&self, block: &BlockMsg) -> Result<()> {
        self.request("SyncSubmitBlock", vec![helper::serialize(block)])
            .await
    }

//    async fn sync_incoming_blocks(
//        &self,
//    ) -> Result<(SubscriptionId, NotificationStream<BlockHeader>)> {
//        self.subscribe("SyncIncomingBlocks", vec![]).await
//    }

    async fn sync_mark_bad(&self, bad_cid: &Cid) -> Result<()> {
        self.request("SyncMarkBad", vec![helper::serialize(&CidJsonRef(bad_cid))])
            .await
    }

    async fn sync_check_bad(&self, bad_cid: &Cid) -> Result<String> {
        self.request("SyncCheckBad", vec![helper::serialize(&CidJsonRef(bad_cid))])
            .await
    }
}