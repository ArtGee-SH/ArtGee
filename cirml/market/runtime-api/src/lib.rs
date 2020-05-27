#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

use sp_std::prelude::Vec;

// re-export
pub use cirml_market::{OnSellInfo, OnSellState};

sp_api::decl_runtime_apis! {
    pub trait MarketApi<ArtvenusId, Balance, BlockNumber> where
        ArtvenusId: Codec,
        Balance: Codec,
        BlockNumber: Codec,
    {
        fn on_sell() -> Vec<(ArtvenusId, OnSellInfo<Balance, BlockNumber>)>;
    }
}
