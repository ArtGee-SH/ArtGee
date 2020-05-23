use cryptoindus_runtime::AccountId;
use jsonrpc_core::{Error, ErrorCode};

pub enum CiRpcErr {}

const BASE_ERROR: i64 = 5000;

// impl From<CiRpcErr> for Error {
//     fn from(e: CiRpcErr) -> Self {
//         match e {
//         }
//     }
// }
