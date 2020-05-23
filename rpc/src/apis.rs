use serde_json::Value;

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use cryptoindus_runtime as runtime;
use runtime::AccountId;

#[rpc]
pub trait CiApi<BlockHash> {}
