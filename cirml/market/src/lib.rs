#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};

use sp_runtime::{DispatchError, DispatchResult, Percent, RuntimeDebug};
use sp_std::prelude::Vec;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    traits::{Currency, ExistenceRequirement::KeepAlive},
    IterableStorageMap,
};
use frame_system::{self as system, ensure_signed};

use cirml_artvenuses::{Artvenus, ArtvenusId};

pub trait Trait:
    frame_system::Trait + cirml_artists::Trait + cirml_artvenuses::Trait + cirml_balances::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as cirml_balances::Trait>::Balance,
		ArtvenusId = ArtvenusId<T>,
	{
	    OnSell(AccountId, ArtvenusId, Balance),
	    Deal(AccountId, ArtvenusId, bool),
	}
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        ///
        NotOnSell,
        ///
        AlreadyOnSell,
        ///
        NotCreaterInVirginSell,
        ///
        NotHolderInSell,
        ///
        CantPay,
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OnSellState {
    VirginSell,
    Sell,
    Bidding,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OnSellInfo<Balance, BlockNumber> {
    state: OnSellState,
    price: Balance,
    time: BlockNumber,
}

decl_storage! {
    trait Store for Module<T: Trait> as Market {
        pub Manager get(fn manager) config(manager): T::AccountId;
        pub VirginSellPercent get(fn vergin_sell_percent) config(virgin_sell_percent): Percent;
        pub NormalSellPercent get(fn normal_sell_percent) config(normal_sell_percent): Percent;

        pub VirginSellOut get(fn virgin_sell_out): map hasher(identity) ArtvenusId<T> => Option<()>;
        pub OnSell get(fn on_sell): map hasher(identity) ArtvenusId<T> => Option<OnSellInfo<T::Balance, T::BlockNumber>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight=0]
        pub fn sell(origin, venus_id: ArtvenusId<T>, #[compact] price: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::sell_impl(who, venus_id, price)?;
            Ok(())
        }

        #[weight=0]
        pub fn deal(origin, venus_id: ArtvenusId<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::deal_impl(who, venus_id)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_on_sell(
        venus_id: ArtvenusId<T>,
    ) -> Result<OnSellInfo<T::Balance, T::BlockNumber>, DispatchError> {
        let sell = Self::on_sell(venus_id).ok_or(Error::<T>::NotOnSell)?;
        Ok(sell)
    }
}

impl<T: Trait> Module<T> {
    fn sell_impl(who: T::AccountId, venus_id: ArtvenusId<T>, price: T::Balance) -> DispatchResult {
        let artvenus: Artvenus<T> = cirml_artvenuses::Module::<T>::get_artvenus(venus_id)?;
        let artist_id = artvenus.origin;

        if Self::get_on_sell(venus_id).is_ok() {
            Err(Error::<T>::AlreadyOnSell)?;
        }
        let state = if Self::virgin_sell_out(&venus_id).is_none() {
            // artist accountid may be changed, thus must get every time
            let artist_account = cirml_artists::Module::<T>::get_artist_account(artist_id)?;
            // virgin sell
            if who != artist_account {
                Err(Error::<T>::NotCreaterInVirginSell)?;
            }
            OnSellState::VirginSell
        } else {
            let seller = cirml_artvenuses::Module::<T>::holder_for(venus_id)?;
            if seller != who {
                Err(Error::<T>::NotHolderInSell)?;
            }
            OnSellState::Sell
        };

        // put sell order
        let sell = OnSellInfo {
            state,
            price,
            time: system::Module::<T>::block_number(),
        };
        OnSell::<T>::insert(&venus_id, sell);

        Self::deposit_event(RawEvent::OnSell(who, venus_id, price));
        Ok(())
    }

    fn deal_impl(buyer: T::AccountId, venus_id: ArtvenusId<T>) -> DispatchResult {
        let sell_info = Self::get_on_sell(venus_id)?;
        let venus_info = cirml_artvenuses::Module::<T>::get_artvenus(venus_id)?;
        let artist = cirml_artists::Module::<T>::get_artist_account(venus_info.origin)?;

        let price = sell_info.price;
        let free = cirml_balances::Module::<T>::free_balance(&buyer);
        if free < price {
            Err(Error::<T>::CantPay)?;
        }

        let is_virgin_sell = if Self::virgin_sell_out(&venus_id).is_none() {
            let manager = Self::manager();
            // virgin sell
            let for_artist_percent = Self::vergin_sell_percent();
            let for_artist = for_artist_percent.saturating_reciprocal_mul(price);
            let for_manager = price - for_artist;
            <cirml_balances::Module<T> as Currency<_>>::transfer(
                &buyer, &artist, for_artist, KeepAlive,
            )?;
            <cirml_balances::Module<T> as Currency<_>>::transfer(
                &buyer,
                &manager,
                for_manager,
                KeepAlive,
            )?;
            // set virgin sell finish
            VirginSellOut::<T>::insert(&venus_id, ());
            true
        } else {
            let seller = cirml_artvenuses::Module::<T>::holder_for(venus_id)?;
            // normal sell
            let for_artist_percent = Self::normal_sell_percent();
            let for_artist = for_artist_percent.saturating_reciprocal_mul(price);
            let for_seller = price - for_artist;
            <cirml_balances::Module<T> as Currency<_>>::transfer(
                &buyer, &artist, for_artist, KeepAlive,
            )?;
            <cirml_balances::Module<T> as Currency<_>>::transfer(
                &buyer, &seller, for_seller, KeepAlive,
            )?;
            false
        };
        cirml_artvenuses::Module::<T>::move_artvenus(venus_id, &buyer)
            .expect("move_artvenus must success");
        OnSell::<T>::remove(&venus_id);

        Self::deposit_event(RawEvent::Deal(buyer, venus_id, is_virgin_sell));
        Ok(())
    }
}

// for runtime-api
impl<T: Trait> Module<T> {
    pub fn on_sell_list() -> Vec<(ArtvenusId<T>, OnSellInfo<T::Balance, T::BlockNumber>)> {
        OnSell::<T>::iter().collect()
    }
}
