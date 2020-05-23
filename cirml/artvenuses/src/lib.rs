#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use sp_runtime::{
    traits::{
        CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize, Member,
        SimpleBitOps,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{fmt::Debug, prelude::*};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, IterableStorageDoubleMap,
    IterableStorageMap, Parameter,
};
use frame_system::{self as system, ensure_signed};

use ci_primitives::{ArtistId, Text};

pub type ArtvenusId<T> = <T as Trait>::Hash;
pub type Artvenus<T> = ArtvenusInfo<<T as frame_system::Trait>::BlockNumber>;

pub trait Trait: frame_system::Trait + cirml_balances::Trait + cirml_artists::Trait {
    /// Art hash
    type Hash: Parameter
        + Member
        + MaybeSerializeDeserialize
        + Debug
        + MaybeDisplay
        + SimpleBitOps
        + Ord
        + Default
        + Copy
        + CheckEqual
        + sp_std::hash::Hash
        + AsRef<[u8]>
        + AsMut<[u8]>
        + MaybeMallocSizeOf;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		ArtvenusId = ArtvenusId<T>
	{
	    Create(ArtistId, ArtvenusId),
	    Move(ArtvenusId, AccountId, AccountId),
	}
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        ///
        ArtvenusNotExist,
        ///
        ArtvenusAlreadyExist,
        ///
        HolderNotExist,
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ArtvenusInfo<BlockNumber> {
    pub origin: ArtistId,
    pub time: BlockNumber,
    pub name: Text,
    pub desc: Text,
}

decl_storage! {
    trait Store for Module<T: Trait> as Artvenuses {
        pub ArtvenusInfos get(fn artvenus_infos): map hasher(identity) ArtvenusId<T> => Option<Artvenus<T>>;

        pub ArtistArtvenuses get(fn artist_artvenuses):
            double_map hasher(twox_64_concat) ArtistId, hasher(twox_64_concat) u64 => Option<ArtvenusId<T>>;
        pub ArtistArtvenusNumbers get(fn artist_artvenus_numbers): map hasher(twox_64_concat) ArtistId => u64;

        pub Holder get(fn holder): map hasher(identity) ArtvenusId<T> => Option<(T::AccountId, u64)>;
        pub HolderArtvenuses get(fn holder_artvenuses):
            double_map hasher(blake2_128_concat) T::AccountId, hasher(twox_64_concat) u64 => Option<ArtvenusId<T>>;
        pub HolderArtvenusNumbers get(fn holder_artvenus_numbers): map hasher(blake2_128_concat) T::AccountId => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight=0]
        pub fn create_artvenus(origin, id: ArtvenusId<T>, name: Text, desc: Text) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::create_artvenus_impl(who, id, name, desc)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_artvenus(hash: ArtvenusId<T>) -> Result<Artvenus<T>, DispatchError> {
        let info = Self::artvenus_infos(hash).ok_or(Error::<T>::ArtvenusNotExist)?;
        Ok(info)
    }

    pub fn artvenus_ids_for(artist_id: ArtistId) -> Vec<ArtvenusId<T>> {
        ArtistArtvenuses::<T>::iter_prefix(artist_id)
            .map(|(_, id)| id)
            .collect()
    }

    pub fn artvenus_for(artist_id: ArtistId) -> Result<Vec<Artvenus<T>>, DispatchError> {
        let ids = Self::artvenus_ids_for(artist_id);
        let mut v = vec![];
        for id in ids {
            v.push(Self::get_artvenus(id)?);
        }
        Ok(v)
    }

    pub fn holder_info_for(venus_id: ArtvenusId<T>) -> Result<(T::AccountId, u64), DispatchError> {
        let r = Self::holder(&venus_id).ok_or(Error::<T>::HolderNotExist)?;
        Ok(r)
    }

    pub fn holder_for(venus_id: ArtvenusId<T>) -> Result<T::AccountId, DispatchError> {
        Self::holder_info_for(venus_id).map(|(a, _)| a)
    }

    pub fn is_holder(venus_id: ArtvenusId<T>, who: &T::AccountId) -> Result<bool, DispatchError> {
        let (source, _) = Self::holder_info_for(venus_id)?;
        Ok(&source == who)
    }
}

impl<T: Trait> Module<T> {
    pub fn create_artvenus_impl(
        who: T::AccountId,
        id: ArtvenusId<T>,
        name: Text,
        desc: Text,
    ) -> DispatchResult {
        let artist_id = cirml_artists::Module::<T>::get_artist_id(&who)?;
        if Self::get_artvenus(id).is_ok() {
            Err(Error::<T>::ArtvenusAlreadyExist)?;
        }
        let info = ArtvenusInfo {
            origin: artist_id,
            time: system::Module::<T>::block_number(),
            name,
            desc,
        };
        let number_for_artist = Self::artist_artvenus_numbers(artist_id);
        // artvenus origin
        ArtvenusInfos::<T>::insert(id, info);
        ArtistArtvenuses::<T>::insert(artist_id, number_for_artist, id);
        ArtistArtvenusNumbers::insert(artist_id, number_for_artist + 1);
        // artvenus relationship init
        let number_for_holder = Self::holder_artvenus_numbers(&who);
        Holder::<T>::insert(&id, (who.clone(), number_for_holder));
        HolderArtvenuses::<T>::insert(&who, number_for_holder, id);
        HolderArtvenusNumbers::<T>::insert(who, number_for_holder + 1);

        Self::deposit_event(RawEvent::Create(artist_id, id));
        Ok(())
    }

    pub fn move_artvenus(id: ArtvenusId<T>, to: &T::AccountId) -> DispatchResult {
        let _ = Self::get_artvenus(id)?;
        let (source, source_number) = Self::holder_info_for(id)?;
        if source == *to {
            // same holder, do nothing
            return Ok(());
        }

        let current_number_for_to = Self::holder_artvenus_numbers(to);

        // remove current relationship for source
        HolderArtvenuses::<T>::remove(&source, source_number);
        // add new relationship for to
        HolderArtvenuses::<T>::insert(to, current_number_for_to, id);
        // override relationship
        Holder::<T>::insert(&id, (to.clone(), current_number_for_to));
        // update index number for to
        HolderArtvenusNumbers::<T>::insert(to, current_number_for_to + 1);

        Self::deposit_event(RawEvent::Move(id, source, to.clone()));
        Ok(())
    }
}

// for runtime-api
impl<T: Trait> Module<T> {
    pub fn artvenuses() -> Vec<ArtvenusId<T>> {
        ArtvenusInfos::<T>::iter().map(|(id, _)| id).collect()
    }

    pub fn artvenuses_of_artist(artist_id: ArtistId) -> Vec<(u64, ArtvenusId<T>)> {
        ArtistArtvenuses::<T>::iter_prefix(artist_id).collect()
    }

    pub fn artvenuses_of_holder(account_id: &T::AccountId) -> Vec<(u64, ArtvenusId<T>)> {
        HolderArtvenuses::<T>::iter_prefix(account_id).collect()
    }
}
