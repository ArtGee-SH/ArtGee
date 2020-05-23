#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

use sp_std::prelude::Vec;

use ci_primitives::ArtistId;

sp_api::decl_runtime_apis! {
    pub trait ArtistsApi<AccountId> where
        AccountId: Codec,
    {
        fn artists() -> Vec<(ArtistId, AccountId)>;
    }
}
