use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{Cid, CidJson, CidJsonRef, StartDealParams, DealInfo, QueryOffer, RetrievalOrder, PeerId,
Address, PeerIdRefWrapper, CommPRet, Import, FileRef};

/// The Client methods all have to do with interacting with the storage and retrieval markets as a client.
#[async_trait::async_trait]
pub trait ClientApi: JsonApi {
    // ClientImport imports file under the specified path into filestore
    async fn client_import(&self, r#ref: &FileRef) -> Result<Cid> {
        let cid: CidJson = self.request("ClientImport", vec![helper::serialize(r#ref)])
            .await?;
        Ok(cid.0)
    }


    async fn client_start_deal(&self, params: &StartDealParams) -> Result<Cid> {
        let cid: CidJson = self.request("ClientStartDeal", vec![helper::serialize(params)])
            .await?;
        Ok(cid.0)
    }

    // return the latest information about a given deal.
    async fn client_get_deal_info(&self, cid: &Cid) -> Result<DealInfo> {
        self.request("ClientGetDealInfo", vec![helper::serialize(&CidJsonRef(cid))])
            .await
    }

    async fn client_list_deals(&self) -> Result<Vec<DealInfo>> {
        self.request("ClientListDeals", vec![]).await
    }


    async fn client_has_local(&self, root: &Cid) -> Result<bool> {
        self.request("ClientHasLocal", vec![helper::serialize(&CidJsonRef(root))])
            .await
    }

    async fn client_find_data(&self, root: &Cid) -> Result<Vec<QueryOffer>> {
        self.request("ClientFindData", vec![helper::serialize(&CidJsonRef(root))])
            .await
    }

    async fn client_retrieve(&self, order: &RetrievalOrder, r#ref: &FileRef) -> Result<()> {
        self.request(
            "ClientFindData",
            vec![helper::serialize(order), helper::serialize(r#ref)],
        )
            .await
    }


//    async fn client_query_ask(
//        &self,
//        peer_id: &PeerId,
//        miner: &Address,
//    ) -> Result<storagemarket::SignedStorageAsk> {
//        self.request(
//            "ClientQueryAsk",
//            vec![
//                helper::serialize(&PeerIdRefWrapper(peer_id)),
//                helper::serialize(miner),
//            ],
//        )
//        .await
//    }


    async fn client_calc_comm_p(&self, inpath: &str, miner: &Address) -> Result<CommPRet> {
        self.request(
            "ClientCalcCommP",
            vec![helper::serialize(&inpath), helper::serialize(miner)],
        )
            .await
    }

    async fn client_gen_car(&self, r#ref: &FileRef, outpath: &str) -> Result<()> {
        self.request(
            "ClientCalcCommP",
            vec![helper::serialize(r#ref), helper::serialize(&outpath)],
        )
            .await
    }

    async fn client_list_imports(&self) -> Result<Vec<Import>> {
        self.request("ClientListImports", vec![]).await
    }
}