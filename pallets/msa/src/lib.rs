#![cfg_attr(not(feature = "std"), no_std)]

use common_primitives::msa::AccountProvider;
use frame_support::{dispatch::DispatchResult, ensure};
use sp_runtime::DispatchError;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use pallet::*;
pub use weights::*;

pub use common_primitives::msa::MessageSenderId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_identifier)]
	pub type MsaIdentifier<T> = StorageValue<_, MessageSenderId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_owner_of)]
	pub type KeyOwnerOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, MessageSenderId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MsaCreated { msa_id: MessageSenderId, key: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		DuplicatedKey,
		MsaIdOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create(10_000))]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (identifier, _) = Self::create_account(who.clone())?;

			Self::deposit_event(Event::MsaCreated { msa_id: identifier, key: who });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn create_account(key: T::AccountId) -> Result<(MessageSenderId, T::AccountId), DispatchError> {
		let next_msa_id = Self::get_next_msa_id()?;
		Self::add_key(next_msa_id, &key)?;
		let _ = Self::set_msa_identifier(next_msa_id);

		Ok((next_msa_id, key))
	}

	pub fn get_next_msa_id() -> Result<MessageSenderId, DispatchError> {
		let next = Self::get_identifier().checked_add(1).ok_or(Error::<T>::MsaIdOverflow)?;

		Ok(next)
	}

	pub fn set_msa_identifier(identifier: MessageSenderId) -> DispatchResult {
		MsaIdentifier::<T>::set(identifier);

		Ok(())
	}

	pub fn add_key(msa_id: MessageSenderId, key: &T::AccountId) -> DispatchResult {
		<KeyOwnerOf<T>>::try_mutate(key, |maybe_msa| {
			ensure!(maybe_msa.is_none(), Error::<T>::DuplicatedKey);

			*maybe_msa = Some(msa_id);

			Ok(())
		})
	}
}

impl<T: Config> AccountProvider for Pallet<T> {
	type AccountId = T::AccountId;
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId> {
		Self::get_owner_of(key)
	}
}
