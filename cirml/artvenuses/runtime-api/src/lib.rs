#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

use sp_std::prelude::Vec;

use ci_primitives::ArtistId;

sp_api::decl_runtime_apis! {
    pub trait ArtvenusesApi<AccountId, ArtvenusId> where
        AccountId: Codec,
        ArtvenusId: Codec,
    {
        fn artvenuses() -> Vec<ArtvenusId>;

        fn artvenuses_of_artist(artist_id: ArtistId) -> Vec<(u64, ArtvenusId)>;

        fn artvenuses_of_holder(account_id: AccountId) -> Vec<(u64, ArtvenusId)>;
    }
}
