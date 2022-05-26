#![cfg_attr(not(feature = "std"), no_std)]

use common_primitives::msa::{AccountProvider, KeyInfoResponse};
use frame_support::{dispatch::DispatchResult, ensure};
pub use pallet::*;
use sp_runtime::{
	traits::{Convert, Verify, Zero},
	DispatchError, MultiSignature,
};

use sp_core::crypto::AccountId32;
pub mod types;
pub use types::{AddKeyData, AddProvider, Delegator, KeyInfo, Provider, ProviderInfo};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
	#[pallet::getter(fn get_provider_info_of)]
	pub type ProviderInfoOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Provider,
		Blake2_128Concat,
		Delegator,
		ProviderInfo<T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_key_info)]
	pub type KeyInfoOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, KeyInfo<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_msa_keys)]
	pub(super) type MsaKeysOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		MessageSenderId,
		BoundedVec<T::AccountId, T::MaxKeys>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Message Service Account was created with a new MessageSenderId
		MsaCreated { msa_id: MessageSenderId, key: T::AccountId },
		/// An AccountId has been associated with a MessageSenderId
		KeyAdded { msa_id: MessageSenderId, key: T::AccountId },
		/// An AccountId was disassociated with its MessageSenderId
		KeyRevoked { key: T::AccountId },
		/// A delegation relationship was added with the given provider and delegator
		ProviderAdded { provider: Provider, delegator: Delegator },
		/// The Delegator revoked its delegation to the Provider
		DelegatorRevokedDelegation { provider: Provider, delegator: Delegator },
		/// The Provider revoked itself as delegate for the Delegator
		ProviderRevokedDelegation { provider: Provider, delegator: Delegator },
	}

	#[pallet::error]
	pub enum Error<T> {
		DuplicatedKey,
		/// MsaId values have reached the maximum
		MsaIdOverflow,
		// Cryptographic signature verification failed for adding a key to MSA
		AddKeySignatureVerificationFailed,
		// Ony the MSA Owner may perform the operation
		NotMsaOwner,
		// Cryptographic signature failed verification
		InvalidSignature,
		// Only the KeyOwner may perform the operation
		NotKeyOwner,
		// An operation was attempted with an unknown Key
		NoKeyExists,
		// An operation was attempted with a revoked Key
		KeyRevoked,
		// The number of key values has reached its maximum
		KeyLimitExceeded,
		// A transaction's Origin AccountId may not revoke itself
		InvalidSelfRevoke,
		// An MSA may not be its own delegate
		InvalidSelfProvider,
		// The delegation relationship already exists for the given MSA Ids
		DuplicateProvider,
		// Cryptographic signature verification failed for adding the Provider as delegate
		AddProviderSignatureVerificationFailed,
		// Origin attempted to add a delegate for someone else's MSA
		UnauthorizedDelegator,
		// Origin attempted to add a different delegate than what was in the payload
		UnauthorizedProvider,
		// The operation was attempted with a revoked delegation
		DelegationRevoked,
		// The operation was attempted with an unknown delegation
		DelegationNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create(10_000))]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (_, _) = Self::create_account(who.clone(), |new_msa_id| -> DispatchResult {
				Self::deposit_event(Event::MsaCreated { msa_id: new_msa_id, key: who });
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::create_sponsored_account_with_delegation())]
		pub fn create_sponsored_account_with_delegation(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			add_provider_payload: AddProvider,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			Self::verify_signature(proof, delegator_key.clone(), add_provider_payload.encode())?;

			let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?.msa_id;
			ensure!(
				add_provider_payload.authorized_msa_id == provider_msa_id,
				Error::<T>::UnauthorizedProvider
			);

			let (_, _) =
				Self::create_account(delegator_key.clone(), |new_msa_id| -> DispatchResult {
					let _ = Self::add_provider(provider_msa_id.into(), new_msa_id.into())?;

					Self::deposit_event(Event::MsaCreated {
						msa_id: new_msa_id.clone(),
						key: delegator_key.clone(),
					});

					Self::deposit_event(Event::ProviderAdded {
						delegator: new_msa_id.into(),
						provider: provider_msa_id.into(),
					});
					Ok(())
				})?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn add_provider_to_msa(
			origin: OriginFor<T>,
			provider_key: T::AccountId,
			proof: MultiSignature,
			add_provider_payload: AddProvider,
		) -> DispatchResult {
			let delegator_key = ensure_signed(origin)?;

			Self::verify_signature(proof, provider_key.clone(), add_provider_payload.encode())
				.map_err(|_| Error::<T>::AddProviderSignatureVerificationFailed)?;

			let payload_authorized_msa_id = add_provider_payload.authorized_msa_id;

			let (provider_msa_id, delegator_msa_id) = Self::ensure_valid_provider(
				&delegator_key,
				&provider_key,
				payload_authorized_msa_id,
			)?;

			let _ = Self::add_provider(provider_msa_id, delegator_msa_id)?;

			Self::deposit_event(Event::ProviderAdded {
				delegator: delegator_msa_id.into(),
				provider: provider_msa_id.into(),
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_msa_delegation_by_delegator(
			origin: OriginFor<T>,
			provider_msa_id: MessageSenderId,
		) -> DispatchResult {
			let delegator_key = ensure_signed(origin)?;

			let delegator_msa_id: Delegator =
				Self::ensure_valid_msa_key(&delegator_key)?.msa_id.into();
			let provider_msa_id = Provider(provider_msa_id);

			Self::revoke_provider(provider_msa_id, delegator_msa_id)?;

			Self::deposit_event(Event::DelegatorRevokedDelegation {
				delegator: delegator_msa_id,
				provider: provider_msa_id,
			});

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
				.map_err(|_| Error::<T>::AddKeySignatureVerificationFailed)?;

			let msa_id = add_key_payload.msa_id;
			Self::ensure_msa_owner(&who, msa_id)?;

			Self::add_key(msa_id, &key.clone(), |new_msa_id| -> DispatchResult {
				Self::deposit_event(Event::KeyAdded { msa_id: new_msa_id, key });
				Ok(())
			})?;

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

		#[pallet::weight((T::WeightInfo::remove_msa_delegation_by_provider(20_000), DispatchClass::Normal, Pays::No))]
		pub fn remove_delegation_by_provider(
			origin: OriginFor<T>,
			delegator: MessageSenderId,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			// Remover should have valid keys (non expired and exists)
			let key_info = Self::ensure_valid_msa_key(&provider_key)?;

			let provider_msa_id = Provider(key_info.msa_id);
			let delegator_msa_id = Delegator(delegator);

			Self::revoke_provider(provider_msa_id, delegator_msa_id)?;

			Self::deposit_event(Event::ProviderRevokedDelegation {
				provider: provider_msa_id,
				delegator: delegator_msa_id,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn create_account<F>(
		key: T::AccountId,
		on_success: F,
	) -> Result<(MessageSenderId, T::AccountId), DispatchError>
	where
		F: FnOnce(MessageSenderId) -> DispatchResult,
	{
		let next_msa_id = Self::get_next_msa_id()?;
		Self::add_key(next_msa_id, &key, on_success)?;
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

	pub fn add_key<F>(msa_id: MessageSenderId, key: &T::AccountId, on_success: F) -> DispatchResult
	where
		F: FnOnce(MessageSenderId) -> DispatchResult,
	{
		KeyInfoOf::<T>::try_mutate(key, |maybe_msa| {
			ensure!(maybe_msa.is_none(), Error::<T>::DuplicatedKey);

			*maybe_msa = Some(KeyInfo {
				msa_id: msa_id.clone(),
				expired: T::BlockNumber::default(),
				nonce: Zero::zero(),
			});

			// adding reverse lookup
			<MsaKeysOf<T>>::try_mutate(msa_id, |key_list| {
				let index = key_list.binary_search(key).err().ok_or(Error::<T>::DuplicatedKey)?;

				key_list
					.try_insert(index, key.clone())
					.map_err(|_| Error::<T>::KeyLimitExceeded)?;

				on_success(msa_id)
			})
		})
	}

	pub fn ensure_valid_provider(
		delegator_key: &T::AccountId,
		provider_key: &T::AccountId,
		authorized_msa_id: MessageSenderId,
	) -> Result<(Provider, Delegator), DispatchError> {
		let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?.msa_id;
		let delegator_msa_id = Self::ensure_valid_msa_key(&delegator_key)?.msa_id;

		ensure!(authorized_msa_id == delegator_msa_id, Error::<T>::UnauthorizedDelegator);

		ensure!(delegator_msa_id != provider_msa_id, Error::<T>::InvalidSelfProvider);

		Ok((provider_msa_id.into(), delegator_msa_id.into()))
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

	pub fn add_provider(provider: Provider, delegator: Delegator) -> DispatchResult {
		ProviderInfoOf::<T>::try_mutate(provider, delegator, |maybe_info| -> DispatchResult {
			ensure!(maybe_info.take() == None, Error::<T>::DuplicateProvider);

			let info = ProviderInfo { permission: Default::default(), expired: Default::default() };

			*maybe_info = Some(info);

			Ok(())
		})?;

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

	pub fn revoke_provider(
		provider_msa_id: Provider,
		delegator_msa_id: Delegator,
	) -> DispatchResult {
		ProviderInfoOf::<T>::try_mutate_exists(
			provider_msa_id,
			delegator_msa_id,
			|maybe_info| -> DispatchResult {
				let mut info = maybe_info.take().ok_or(Error::<T>::DelegationNotFound)?;

				ensure!(info.expired == T::BlockNumber::default(), Error::<T>::DelegationRevoked);

				let current_block = frame_system::Pallet::<T>::block_number();

				info.expired = current_block;

				*maybe_info = Some(info);

				Ok(())
			},
		)?;

		Ok(())
	}

	pub fn try_get_key_info(key: &T::AccountId) -> Result<KeyInfo<T::BlockNumber>, DispatchError> {
		let info = Self::get_key_info(key).ok_or(Error::<T>::NoKeyExists)?;
		Ok(info)
	}

	pub fn get_owner_of(key: &T::AccountId) -> Option<MessageSenderId> {
		Self::get_key_info(&key).map(|info| info.msa_id)
	}

	/// Fetches all the keys associated with a message Source Account
	/// NOTE: This should only be called from RPC due to heavy database reads
	pub fn fetch_msa_keys(
		msa_id: MessageSenderId,
	) -> Vec<KeyInfoResponse<T::AccountId, T::BlockNumber>> {
		let mut response = Vec::new();
		for key in Self::get_msa_keys(msa_id) {
			if let Ok(info) = Self::try_get_key_info(&key) {
				response.push(info.map_to_response(key));
			}
		}

		response
	}

	pub fn ensure_valid_msa_key(
		key: &T::AccountId,
	) -> Result<KeyInfo<T::BlockNumber>, DispatchError> {
		let info = Self::try_get_key_info(key)?;

		ensure!(info.expired == T::BlockNumber::zero(), Error::<T>::KeyRevoked);

		Ok(info)
	}
}

impl<T: Config> AccountProvider for Pallet<T> {
	type AccountId = T::AccountId;
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId> {
		Self::get_owner_of(key)
	}
}
