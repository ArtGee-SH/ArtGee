#![allow(unused)]

#[macro_use]
mod utils;

mod apis;
mod errors;
mod impls;
mod types;

use std::fmt;
use std::sync::Arc;

use sc_client_api::{backend::Backend, CallExecutor, StorageProvider};
use sc_service::client::Client;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;

use cryptoindus_runtime::{
    opaque::Block, AccountId, ArtvenusId, Balance, BlockNumber, Index, UncheckedExtrinsic,
};

use cirml_artists_rpc::{Artists, ArtistsApi};
use cirml_artvenuses_rpc::{Artvenuses, ArtvenusesApi};
use cirml_market_rpc::{Market, MarketApi};

use apis::CiApi;
use impls::CiRpc;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

/// Full client dependencies.
pub struct FullDeps<P, BE, E, RA> {
    /// The client instance to use.
    pub client: Arc<Client<BE, E, Block, RA>>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<P, M, BE, E, RA>(deps: FullDeps<P, BE, E, RA>) -> jsonrpc_core::IoHandler<M>
where
    BE: Backend<Block> + 'static,
    BE::State: sp_state_machine::backend::Backend<sp_runtime::traits::BlakeTwo256>,
    E: CallExecutor<Block> + Clone + Send + Sync,
    RA: Send + Sync + 'static,
    // B: BlockT + 'static,
    Client<BE, E, Block, RA>: ProvideRuntimeApi<Block>,
    Client<BE, E, Block, RA>: HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + StorageProvider<Block, BE>
        + 'static,
    Client<BE, E, Block, RA>: Send + Sync + 'static,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<
            Block,
            Balance,
            UncheckedExtrinsic,
        >,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        cirml_artists_runtime_api::ArtistsApi<Block, AccountId>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        cirml_artvenuses_runtime_api::ArtvenusesApi<Block, AccountId, ArtvenusId>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        cirml_market_runtime_api::MarketApi<Block, ArtvenusId, Balance, BlockNumber>,
    <<Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api as sp_api::ApiErrorExt>::Error:
        fmt::Debug,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps { client, pool } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool,
    )));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    // cirml
    io.extend_with(ArtistsApi::to_delegate(Artists::new(client.clone())));
    io.extend_with(ArtvenusesApi::to_delegate(Artvenuses::new(client.clone())));
    io.extend_with(MarketApi::to_delegate(Market::new(client.clone())));

    io.extend_with(CiApi::to_delegate(CiRpc::new(client)));

    io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(deps: LightDeps<C, F, P>) -> jsonrpc_core::IoHandler<M>
where
    C: sc_client_api::blockchain::HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: sc_client_api::light::Fetcher<Block> + 'static,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use substrate_frame_rpc_system::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher,
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(SystemApi::<AccountId, Index>::to_delegate(
        LightSystem::new(client, remote_blockchain, fetcher, pool),
    ));

    io
}
