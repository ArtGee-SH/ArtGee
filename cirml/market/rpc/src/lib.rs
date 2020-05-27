use std::collections::HashMap;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::Serialize;

use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use cirml_market_runtime_api::{MarketApi as MarketRuntimeApi, OnSellInfo, OnSellState};

pub struct Market<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Market<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Market {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait MarketApi<BlockHash, ArtvenusId, Balance, BlockNumber> {
    #[rpc(name = "market_getOnSells")]
    fn on_sell(&self, at: Option<BlockHash>) -> Result<serde_json::Value>;
}

impl<C, Block, ArtvenusId, Balance, BlockNumber>
    MarketApi<<Block as BlockT>::Hash, ArtvenusId, Balance, BlockNumber> for Market<C, Block>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: MarketRuntimeApi<Block, ArtvenusId, Balance, BlockNumber>,
    Block: BlockT,
    ArtvenusId:
        Clone + std::hash::Hash + std::cmp::Eq + std::fmt::Display + Codec + serde::Serialize,
    Balance: Clone + std::fmt::Display + Codec + serde::Serialize + ToString,
    BlockNumber: Clone + std::fmt::Display + Codec + serde::Serialize,
{
    fn on_sell(&self, at: Option<<Block as BlockT>::Hash>) -> Result<serde_json::Value> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let r: HashMap<ArtvenusId, OnSellInfoForRpc<BlockNumber>> = api
            .on_sell(&at)
            .map(|list| {
                list.into_iter()
                    .map(|(id, info)| (id, info.into()))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)?;
        let map = serde_json::value::to_value(r).map_err(serde_error_into_rpc_err)?;
        Ok(map)
    }
}

#[derive(Serialize)]
struct OnSellInfoForRpc<BlockNumber> {
    state: OnSellState,
    price: String,
    time: BlockNumber,
}

impl<BlockNumber, Balance> From<OnSellInfo<Balance, BlockNumber>> for OnSellInfoForRpc<BlockNumber>
where
    Balance: ToString,
{
    fn from(runtime_info: OnSellInfo<Balance, BlockNumber>) -> Self {
        OnSellInfoForRpc {
            state: runtime_info.state,
            price: runtime_info.price.to_string(),
            time: runtime_info.time,
        }
    }
}

// TODO remove in future
const RUNTIME_ERROR: i64 = 1;
const SERDE_JSON_ERROR: i64 = 2;

fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

fn serde_error_into_rpc_err(err: serde_json::Error) -> Error {
    Error {
        code: ErrorCode::ServerError(SERDE_JSON_ERROR),
        message: "Serialize data error".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
