use std::collections::BTreeMap;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use ci_primitives::ArtistId;
use cirml_artists_runtime_api::ArtistsApi as ArtistsRuntimeApi;

pub struct Artists<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Artists<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Artists {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait ArtistsApi<BlockHash, AccountId> {
    #[rpc(name = "artists")]
    fn artists(&self, at: Option<BlockHash>) -> Result<BTreeMap<ArtistId, AccountId>>;
}

impl<C, Block, AccountId> ArtistsApi<<Block as BlockT>::Hash, AccountId> for Artists<C, Block>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: ArtistsRuntimeApi<Block, AccountId>,
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
{
    fn artists(&self, at: Option<<Block as BlockT>::Hash>) -> Result<BTreeMap<u32, AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.artists(&at)
            .map(|list| list.into_iter().collect())
            .map_err(runtime_error_into_rpc_err)
    }
}

const RUNTIME_ERROR: i64 = 1;
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
