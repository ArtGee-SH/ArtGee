#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
    traits::{
        CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize, Member,
        SimpleBitOps,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::fmt::Debug;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, Parameter};

pub trait Trait: frame_system::Trait + cirml_balances::Trait {
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
	{
	    Create(AccountId, AccountId),
	    Move(AccountId, AccountId),
	}
);

decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Artvenus {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}
