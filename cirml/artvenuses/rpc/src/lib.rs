use std::collections::BTreeMap;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use ci_primitives::ArtistId;
use cirml_artvenuses_runtime_api::ArtvenusesApi as ArtvenusesRuntimeApi;

pub struct Artvenuses<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Artvenuses<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Artvenuses {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait ArtvenusesApi<BlockHash, AccountId, ArtvenusId> {
    #[rpc(name = "artvenuses")]
    fn artvenuses(&self, at: Option<BlockHash>) -> Result<Vec<ArtvenusId>>;

    #[rpc(name = "artvenusesByArtist")]
    fn artvenuses_of_artist(
        &self,
        artist_id: ArtistId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<u64, ArtvenusId>>;

    #[rpc(name = "artvenusesByHolder")]
    fn artvenuses_of_holder(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<u64, ArtvenusId>>;
}

impl<C, Block, AccountId, ArtvenusId> ArtvenusesApi<<Block as BlockT>::Hash, AccountId, ArtvenusId>
    for Artvenuses<C, Block>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: ArtvenusesRuntimeApi<Block, AccountId, ArtvenusId>,
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
    ArtvenusId: Clone + std::fmt::Display + Codec,
{
    fn artvenuses(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<ArtvenusId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.artvenuses(&at).map_err(runtime_error_into_rpc_err)
    }

    fn artvenuses_of_artist(
        &self,
        artist_id: ArtistId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u64, ArtvenusId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.artvenuses_of_artist(&at, artist_id)
            .map(|list| list.into_iter().collect())
            .map_err(runtime_error_into_rpc_err)
    }

    fn artvenuses_of_holder(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u64, ArtvenusId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.artvenuses_of_holder(&at, account_id)
            .map(|list| list.into_iter().collect())
            .map_err(runtime_error_into_rpc_err)
    }
}

// TODO remove in future
const RUNTIME_ERROR: i64 = 1;
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
