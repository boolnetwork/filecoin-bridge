use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{Bytes, TipSet, DomainSeparationTag, ChainEpoch, Randomness, BytesRef, TipSetKey,
                   BlockHeader, CidJsonRef, BlockMessages, MessageReceipt, ParentMessage, ObjStat,
                   BigIntWrapper, UnsignedMessage, HeadChange, BigInt, Cid};
use forest_blocks::{self, tipset::tipset_json::TipsetJson};
use rpc::BlockMessages as forest_BlockMessages;
#[async_trait::async_trait]
pub trait ChainApi: JsonApi {

//    async fn chain_notify(&self) -> Result<(SubscriptionId, NotificationStream<Vec<HeadChange>>)> {
//        self.subscribe("ChainNotify", vec![]).await
//    }

    async fn chain_head(&self) -> Result<TipsetJson> {
        self.request("ChainHead", vec![]).await
    }

    async fn chain_get_randomness(
        &self,
        key: &TipSetKey,
        personalization: &DomainSeparationTag,
        rand_epoch: ChainEpoch,
        entropy: &[u8],
    ) -> Result<Randomness> {
        self.request(
            "ChainGetRandomness",
            vec![
                helper::serialize(key),
                helper::serialize(personalization),
                helper::serialize(&rand_epoch),
                helper::serialize(&BytesRef::from(entropy)),
            ],
        )
            .await
    }

    async fn chain_get_block(&self, cid: &Cid) -> Result<BlockHeader> {
        self.request("ChainGetBlock", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn chain_get_tipset(&self, key: &TipSetKey) -> Result<TipSet> {
        self.request("ChainGetTipSet", vec![helper::serialize(key)])
            .await
    }

    async fn chain_get_block_messages(&self, cid: &forest_cid::Cid) -> Result<forest_BlockMessages> {
        self.request("ChainGetBlockMessages", vec![helper::serialize(&forest_cid::json::CidJson(cid.clone()))])
            .await
    }

    async fn chain_get_parent_receipts(&self, cid: &Cid) -> Result<Vec<MessageReceipt>> {
        self.request("ChainGetParentReceipts", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn chain_get_parent_messages(&self, cid: &Cid) -> Result<Vec<ParentMessage>> {
        self.request("ChainGetParentMessages", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn chain_get_tipset_by_height(
        &self,
        height: ChainEpoch,
        key: &TipSetKey,
    ) -> Result<TipSet> {
        self.request(
            "ChainGetTipSetByHeight",
            vec![helper::serialize(&height), helper::serialize(key)],
        )
            .await
    }

    async fn chain_read_obj(&self, cid: &Cid) -> Result<Vec<u8>> {
        let bytes: Bytes = self
            .request("ChainReadObj", vec![helper::serialize(&CidJsonRef(cid))])
            .await?;
        Ok(bytes.into_inner())
    }

    async fn chain_has_obj(&self, cid: &Cid) -> Result<bool> {
        self.request("ChainHasObj", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn chain_stat_obj(&self, obj: &Cid, base: &Cid) -> Result<ObjStat> {
        self.request(
            "ChainHasObj",
            vec![helper::serialize(&CidJsonRef(obj)), helper::serialize(&CidJsonRef(base))],
        )
            .await
    }

    async fn chain_set_head(&self, key: &TipSetKey) -> Result<()> {
        self.request("ChainSetHead", vec![helper::serialize(key)])
            .await
    }

    async fn chain_get_genesis(&self) -> Result<TipSet> {
        self.request("ChainGetGenesis", vec![]).await
    }

    async fn chain_tipset_weight(&self, key: &TipSetKey) -> Result<BigInt> {
        let bigint: BigIntWrapper = self
            .request("ChainTipSetWeight", vec![helper::serialize(key)])
            .await?;
        Ok(bigint.0)
    }

    async fn chain_get_message(&self, cid: &Cid) -> Result<UnsignedMessage> {
        self.request("ChainGetMessage", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn chain_get_path(&self, from: &TipSetKey, to: &TipSetKey) -> Result<Vec<HeadChange>> {
        self.request(
            "ChainGetPath",
            vec![helper::serialize(from), helper::serialize(to)],
        )
            .await
    }

//    async fn chain_export(
//        &self,
//        key: &TipsetKey,
//    ) -> Result<(SubscriptionId, NotificationStream<Bytes>)> {
//        self.subscribe("ChainExport", vec![helper::serialize(key)])
//            .await
//    }
}