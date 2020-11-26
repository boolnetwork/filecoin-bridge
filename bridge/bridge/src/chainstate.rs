use sp_runtime::{generic::{BlockId ,Era}, traits::{Block as BlockT, Zero}};
use sc_client_api::{BlockchainEvents, backend, notifications::StorageEventStream};
use sc_block_builder::{BlockBuilderProvider};
use sp_blockchain::{HeaderBackend};
use sp_api::{ProvideRuntimeApi, CallApiAt};
use std::{sync::Arc, u64, marker::PhantomData, thread};
use filecoin_bridge_runtime::{apis::VendorApi};

#[derive(Clone)]
pub struct ChainState<Block,B,C>
    where
        Block: BlockT,
        B: backend::Backend<Block> + Send + Sync + 'static,
        C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
        + CallApiAt<Block> + Send + Sync + 'static,
        C::Api: VendorApi<Block>,
{
    pub client: Arc<C>,
    _phantom: PhantomData<(Block,B)>,
}

impl <Block,B,C>ChainState<Block,B,C>
    where
        Block: BlockT,
        B: backend::Backend<Block> + Send + Sync + 'static,
        C: BlockBuilderProvider<B, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockchainEvents<Block>
        + CallApiAt<Block> + Send + Sync + 'static,
        C::Api: VendorApi<Block>,
{
    pub fn new(client:Arc<C>) -> Self{
        ChainState{
            client:client,
            _phantom: PhantomData,
        }
    }

    pub fn tss_pubkey(&self) -> Vec<u8> {
        let info = self.client.info();
        let at: BlockId<Block> = BlockId::Hash(info.best_hash);

        self.client
            .runtime_api()
            .tss_pub_key(&at)
            .unwrap()
    }
}