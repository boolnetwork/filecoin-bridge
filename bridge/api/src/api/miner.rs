use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{TipSetKey, Address, ChainEpoch, MiningBaseInfo, BlockTemplate, BlockMsg};

#[async_trait::async_trait]
pub trait MinerApi: JsonApi {
    async fn miner_get_base_info(
        &self,
        addr: &Address,
        height: ChainEpoch,
        key: &TipSetKey,
    ) -> Result<Option<MiningBaseInfo>> {
        self.request(
            "MinerGetBaseInfo",
            vec![
                helper::serialize(addr),
                helper::serialize(&height),
                helper::serialize(key),
            ],
        )
            .await
    }

    async fn miner_create_block(&self, template: &BlockTemplate) -> Result<BlockMsg> {
        self.request("MinerCreateBlock", vec![helper::serialize(template)])
            .await
    }
}