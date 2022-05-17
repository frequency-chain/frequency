#![cfg_attr(not(feature = "std"), no_std)]

use common_primitives::msa::AccountProvider;
use frame_support::{dispatch::DispatchResult, ensure};
pub use pallet::*;
use sp_runtime::{
	traits::{Convert, Verify, Zero},
	DispatchError, MultiSignature,
};

use sp_core::crypto::AccountId32;

pub mod types;
pub use types::{AddKeyData, KeyInfo};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use weights::*;

pub use common_primitives::{msa::MessageSenderId, utils::wrap_binary_data};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		#[pallet::constant]
		type MaxKeys: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_identifier)]
	pub type MsaIdentifier<T> = StorageValue<_, MessageSenderId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_key_info)]
	pub type KeyInfoOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, KeyInfo<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_msa_keys)]
	pub(super) type MsaKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		MessageSenderId,
		BoundedVec<T::AccountId, T::MaxKeys>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MsaCreated { msa_id: MessageSenderId, key: T::AccountId },
		KeyAdded { msa_id: MessageSenderId, key: T::AccountId },
		KeyRevoked { key: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		DuplicatedKey,
		MsaIdOverflow,
		KeyVerificationFailed,
		NotMsaOwner,
		InvalidSignature,
		NotKeyOwner,
		NoKeyExists,
		KeyRevoked,
		KeyLimitExceeded,
		InvalidSelfRevoke,
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

		#[pallet::weight(10_000)]
		pub fn add_key_to_msa(
			origin: OriginFor<T>,
			key: T::AccountId,
			proof: MultiSignature,
			add_key_payload: AddKeyData,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::verify_signature(proof, key.clone(), add_key_payload.encode())
				.map_err(|_| Error::<T>::KeyVerificationFailed)?;

			let msa_id = add_key_payload.msa_id;
			Self::ensure_msa_owner(&who, msa_id)?;

			Self::add_key(msa_id, &key)?;

			Self::deposit_event(Event::KeyAdded { msa_id, key });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_msa_key(origin: OriginFor<T>, key: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(who != key, Error::<T>::InvalidSelfRevoke);

			let who = Self::try_get_key_info(&who)?;
			let key_info = Self::try_get_key_info(&key)?;
			ensure!(who.expired == T::BlockNumber::zero(), Error::<T>::KeyRevoked);
			ensure!(who.msa_id == key_info.msa_id, Error::<T>::NotKeyOwner);

			Self::revoke_key(&key)?;

			Self::deposit_event(Event::KeyRevoked { key });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn create_account(
		key: T::AccountId,
	) -> Result<(MessageSenderId, T::AccountId), DispatchError> {
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
		KeyInfoOf::<T>::try_mutate(key, |maybe_msa| {
			ensure!(maybe_msa.is_none(), Error::<T>::DuplicatedKey);

			*maybe_msa =
				Some(KeyInfo { msa_id, expired: T::BlockNumber::default(), nonce: Zero::zero() });

			// adding reverse lookup
			<MsaKeys<T>>::try_mutate(msa_id, |ref mut key_list| {
				let index = key_list.binary_search(key).err().ok_or(Error::<T>::DuplicatedKey)?;

				key_list
					.try_insert(index, key.clone())
					.map_err(|_| Error::<T>::KeyLimitExceeded)?;

				Ok(())
			})
		})
	}

	pub fn ensure_msa_owner(who: &T::AccountId, msa_id: MessageSenderId) -> DispatchResult {
		let signer_msa_id = Self::get_owner_of(who).ok_or(Error::<T>::NoKeyExists)?;

		ensure!(signer_msa_id == msa_id, Error::<T>::NotMsaOwner);

		Ok(())
	}

	pub fn verify_signature(
		signature: MultiSignature,
		signer: T::AccountId,
		payload: Vec<u8>,
	) -> DispatchResult {
		let key = T::ConvertIntoAccountId32::convert(signer.clone());
		let wrapped_payload = wrap_binary_data(payload);
		ensure!(
			signature.verify(&wrapped_payload[..], &key.clone().into()),
			Error::<T>::InvalidSignature
		);

		Ok(())
	}

	pub fn revoke_key(key: &T::AccountId) -> DispatchResult {
		KeyInfoOf::<T>::try_mutate(key, |maybe_info| -> DispatchResult {
			let mut info = maybe_info.take().ok_or(Error::<T>::NoKeyExists)?;

			ensure!(info.expired == T::BlockNumber::default(), Error::<T>::KeyRevoked);

			let current_block = frame_system::Pallet::<T>::block_number();

			info.expired = current_block;

			*maybe_info = Some(info);

			Ok(())
		})?;

		Ok(())
	}

	pub fn try_get_key_info(key: &T::AccountId) -> Result<KeyInfo<T::BlockNumber>, DispatchError> {
		let info = Self::get_key_info(key).ok_or(Error::<T>::NoKeyExists)?;
		Ok(info)
	}

	pub fn get_owner_of(key: &T::AccountId) -> Option<MessageSenderId> {
		Self::get_key_info(&key).map(|info| info.msa_id)
	}

	pub fn fetch_msa_keys(msa_id: MessageSenderId) -> Vec<T::AccountId> {
		let keys = Self::get_msa_keys(msa_id);
		keys.into_inner()
	}
}

impl<T: Config> AccountProvider for Pallet<T> {
	type AccountId = T::AccountId;
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId> {
		Self::get_owner_of(key)
	}
}
