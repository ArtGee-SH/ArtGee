#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use sp_runtime::{traits::StaticLookup, DispatchError, DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, IterableStorageMap};
use frame_system::{self as system, ensure_root, ensure_signed};

use ci_primitives::{ArtistId, Text};

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
	{
	    RegisterArtist(ArtistId),
	    BindArtist(AccountId, ArtistId),
	}
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        ///
        NameAlreadyExist,
        ///
        ArtistNotExist,
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, RuntimeDebug)]
pub enum Gender {
    Male,
    Female,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Male
    }
}

pub enum ArtistIdentity<'a, T: Trait> {
    AccountId(&'a T::AccountId),
    Id(ArtistId),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ArtistInfo {
    pub name: Text,
    pub gender: Gender,
}

decl_storage! {
    trait Store for Module<T: Trait> as Artists {
        pub NextArtistId get(fn next_artist_id): u32 = 0;
        pub ArtistIds get(fn artist_ids): map hasher(blake2_128_concat) T::AccountId => Option<ArtistId>;
        pub ArtistAccounts get(fn artist_accounts): map hasher(twox_64_concat) ArtistId => Option<T::AccountId>;
        pub ArtistInfos get(fn artist_infos): map hasher(twox_64_concat) ArtistId => Option<ArtistInfo>;

        pub Names get(fn names): map hasher(blake2_128_concat) Text => Option<()>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 0]
        pub fn regist_artist(origin, who: <T::Lookup as StaticLookup>::Source, artist: ArtistInfo) -> DispatchResult {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;

            if Self::names(&artist.name).is_some() {
                Err(Error::<T>::NameAlreadyExist)?;
            }

            let artist_id = Self::next_artist_id();

            // set storage
            Names::insert(&artist.name, ());
            ArtistIds::<T>::insert(&who, artist_id);
            ArtistAccounts::<T>::insert(&artist_id, who.clone());
            ArtistInfos::insert(artist_id, artist);
            NextArtistId::put(artist_id + 1);

            Self::deposit_event(RawEvent::RegisterArtist(artist_id));
            Self::deposit_event(RawEvent::BindArtist(who, artist_id));

            Ok(())
        }

        #[weight = 0]
        pub fn update_binding(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let source = ensure_signed(origin)?;
            let who = T::Lookup::lookup(who)?;
            let artist_id = Self::get_artist_id(&source)?;

            ArtistIds::<T>::insert(&who, artist_id);
            ArtistAccounts::<T>::insert(&artist_id, who.clone());

            Self::deposit_event(RawEvent::BindArtist(who, artist_id));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_artist_id(who: &T::AccountId) -> Result<ArtistId, DispatchError> {
        let id = Self::artist_ids(who).ok_or(Error::<T>::ArtistNotExist)?;
        Ok(id)
    }

    pub fn get_artist_account(id: ArtistId) -> Result<T::AccountId, DispatchError> {
        let who = Self::artist_accounts(id).ok_or(Error::<T>::ArtistNotExist)?;
        Ok(who)
    }

    pub fn get_artist_info(who: ArtistIdentity<T>) -> Result<ArtistInfo, DispatchError> {
        let artist_id = match who {
            ArtistIdentity::AccountId(account_id) => Self::get_artist_id(account_id)?,
            ArtistIdentity::Id(artist_id) => artist_id,
        };
        let artist = Self::artist_infos(artist_id).ok_or(Error::<T>::ArtistNotExist)?;
        Ok(artist)
    }
}

// for runtime api
impl<T: Trait> Module<T> {
    pub fn artists() -> Vec<(ArtistId, T::AccountId)> {
        ArtistAccounts::<T>::iter().collect()
    }
}
