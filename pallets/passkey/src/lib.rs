//! Using Passkeys to execute transactions
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
use common_primitives::utils::wrap_binary_data;
use common_runtime::extensions::check_nonce::CheckNonce;
use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::Contains,
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::{
	generic::Era,
	traits::{Convert, Dispatchable, SignedExtension, Verify, Zero},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	AccountId32, MultiSignature,
};
use sp_std::{vec, vec::Vec};

/// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;

/// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_common;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::tokens::fungible::Mutate;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// defines all new types for this pallet
pub mod types;
pub use types::*;

pub use module::*;

#[frame_support::pallet]
pub mod module {

	use super::*;

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_transaction_payment::Config + Send + Sync
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ From<Call<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// AccountId truncated to 32 bytes
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		/// Filters the inner calls for passkey which is set in runtime
		type PasskeyCallFilter: Contains<<Self as Config>::RuntimeCall>;

		/// Helper Curreny method for benchmarking
		#[cfg(feature = "runtime-benchmarks")]
		type Currency: Mutate<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// InvalidAccountSignature
		InvalidAccountSignature,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// When a passkey transaction is successfully executed
		TransactionExecutionSuccess {
			/// transaction account id
			account_id: T::AccountId,
		},
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// proxy call
		#[pallet::call_index(0)]
		#[pallet::weight({
			let dispatch_info = payload.passkey_call.call.get_dispatch_info();
			let overhead = T::WeightInfo::pre_dispatch();
			let total = overhead.saturating_add(dispatch_info.weight);
			(total, dispatch_info.class)
		})]
		pub fn proxy(
			origin: OriginFor<T>,
			payload: PasskeyPayload<T>,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let transaction_account_id = payload.passkey_call.account_id.clone();
			let main_origin = T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(
				transaction_account_id.clone(),
			));
			let result = payload.passkey_call.call.dispatch(main_origin);
			if result.is_ok() {
				Self::deposit_event(Event::TransactionExecutionSuccess {
					account_id: transaction_account_id,
				});
			}
			result
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		BalanceOf<T>: Send + Sync + From<u64>,
		<T as frame_system::Config>::RuntimeCall:
			From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	{
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = ValidTransaction::default();
			let payload = Self::filter_valid_calls(&call)?;

			let frame_system_checks =
				FrameSystemChecks(payload.passkey_call.account_id.clone(), call.clone());
			let frame_system_validity = frame_system_checks.validate()?;

			let signatures_check = PasskeySignatureCheck::new(payload.clone());
			let signature_validity = signatures_check.validate()?;

			let nonce_check = PasskeyNonceCheck::new(payload.passkey_call.clone());
			let nonce_validity = nonce_check.validate()?;

			let tx_charge = ChargeTransactionPayment::<T>(
				payload.passkey_call.account_id.clone(),
				call.clone(),
			);
			let tx_payment_validity = tx_charge.validate()?;

			let valid_tx = valid_tx
				.combine_with(frame_system_validity)
				.combine_with(signature_validity)
				.combine_with(nonce_validity)
				.combine_with(tx_payment_validity);
			Ok(valid_tx)
		}

		fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
			let payload = Self::filter_valid_calls(&call)?;

			let frame_system_checks =
				FrameSystemChecks(payload.passkey_call.account_id.clone(), call.clone());
			frame_system_checks.pre_dispatch()?;

			let signatures_check = PasskeySignatureCheck::new(payload.clone());
			signatures_check.pre_dispatch()?;

			let nonce_check = PasskeyNonceCheck::new(payload.passkey_call.clone());
			nonce_check.pre_dispatch()?;

			let tx_charge = ChargeTransactionPayment::<T>(
				payload.passkey_call.account_id.clone(),
				call.clone(),
			);
			tx_charge.pre_dispatch()
		}
	}
}

impl<T: Config> Pallet<T>
where
	BalanceOf<T>: Send + Sync + From<u64>,
	<T as frame_system::Config>::RuntimeCall: From<Call<T>> + Dispatchable<Info = DispatchInfo>,
{
	fn filter_valid_calls(call: &Call<T>) -> Result<PasskeyPayload<T>, TransactionValidityError> {
		match call {
			Call::proxy { payload }
				if T::PasskeyCallFilter::contains(&payload.clone().passkey_call.call) =>
				return Ok(payload.clone()),
			_ => return Err(InvalidTransaction::Call.into()),
		}
	}
}

/// Passkey specific nonce check
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
struct PasskeyNonceCheck<T: Config>(pub PasskeyCall<T>);

impl<T: Config> PasskeyNonceCheck<T>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo>,
{
	pub fn new(passkey_call: PasskeyCall<T>) -> Self {
		Self(passkey_call)
	}

	pub fn validate(&self) -> TransactionValidity {
		let who = self.0.account_id.clone();
		let nonce = self.0.account_nonce;
		let some_call: &<T as Config>::RuntimeCall = &self.0.call;
		let info = &some_call.get_dispatch_info();

		let passkey_nonce = CheckNonce::<T>::from(nonce);
		passkey_nonce.validate(&who, &some_call.clone().into(), info, 0usize)
	}

	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let who = self.0.account_id.clone();
		let nonce = self.0.account_nonce;
		let some_call: &<T as Config>::RuntimeCall = &self.0.call;
		let info = &some_call.get_dispatch_info();

		let passkey_nonce = CheckNonce::<T>::from(nonce);
		passkey_nonce.pre_dispatch(&who, &some_call.clone().into(), info, 0usize)
	}
}

/// Passkey signatures check
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
struct PasskeySignatureCheck<T: Config>(pub PasskeyPayload<T>);

impl<T: Config> PasskeySignatureCheck<T> {
	pub fn new(passkey_payload: PasskeyPayload<T>) -> Self {
		Self(passkey_payload)
	}

	pub fn validate(&self) -> TransactionValidity {
		// checking the passkey signature to ensure access to the passkey
		let p256_signed_data = self.0.passkey_call.encode();
		let p256_signature = self.0.verifiable_passkey_signature.clone();
		let p256_signer = self.0.passkey_public_key.clone();

		p256_signature
			.try_verify(&p256_signed_data, &p256_signer)
			.map_err(|e| match e {
				PasskeyVerificationError::InvalidProof =>
					TransactionValidityError::Invalid(InvalidTransaction::BadSigner),
				_ => TransactionValidityError::Invalid(InvalidTransaction::Custom(e.into())),
			})?;

		// checking account signature to verify ownership of the account used
		let signed_data = self.0.passkey_public_key.clone();
		let signature = self.0.passkey_call.account_ownership_proof.clone();
		let signer = &self.0.passkey_call.account_id;

		match Self::check_account_signature(signer, &signed_data.inner().to_vec(), &signature) {
			Ok(_) => Ok(ValidTransaction::default()),
			Err(_e) => InvalidTransaction::BadSigner.into(),
		}
	}

	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let _ = self.validate()?;
		Ok(())
	}

	/// Check the signature on passkey public key by the account id
	/// Returns Ok(()) if the signature is valid
	/// Returns Err(InvalidAccountSignature) if the signature is invalid
	/// # Arguments
	/// * `signer` - The account id of the signer
	/// * `signed_data` - The signed data
	/// * `signature` - The signature
	/// # Return
	/// * `Ok(())` if the signature is valid
	/// * `Err(InvalidAccountSignature)` if the signature is invalid
	fn check_account_signature(
		signer: &T::AccountId,
		signed_data: &Vec<u8>,
		signature: &MultiSignature,
	) -> DispatchResult {
		let key = T::ConvertIntoAccountId32::convert((*signer).clone());
		let signed_payload: Vec<u8> = wrap_binary_data(signed_data.clone().into());

		let verified = signature.verify(&signed_payload[..], &key);
		if verified {
			Ok(())
		} else {
			Err(Error::<T>::InvalidAccountSignature.into())
		}
	}
}

/// Passkey related tx payment
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPayment<T: Config>(pub T::AccountId, pub Call<T>);

impl<T: Config> ChargeTransactionPayment<T>
where
	BalanceOf<T>: Send + Sync + From<u64>,
	<T as frame_system::Config>::RuntimeCall:
		From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	/// Validates the transaction fee paid with tokens.
	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let who = self.0.clone();
		pallet_transaction_payment::ChargeTransactionPayment::<T>::from(Zero::zero())
			.pre_dispatch(&who, &runtime_call, info, len)?;
		Ok(())
	}

	/// Validates the transaction fee paid with tokens.
	pub fn validate(&self) -> TransactionValidity {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let who = self.0.clone();

		pallet_transaction_payment::ChargeTransactionPayment::<T>::from(Zero::zero()).validate(
			&who,
			&runtime_call,
			info,
			len,
		)
	}
}

/// Frame system related checks
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct FrameSystemChecks<T: Config + Send + Sync>(pub T::AccountId, pub Call<T>);

impl<T: Config + Send + Sync> FrameSystemChecks<T>
where
	<T as frame_system::Config>::RuntimeCall:
		From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	/// Validates the transaction fee paid with tokens.
	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let who = self.0.clone();

		let non_zero_sender_check = frame_system::CheckNonZeroSender::<T>::new();
		let spec_version_check = frame_system::CheckSpecVersion::<T>::new();
		let tx_version_check = frame_system::CheckTxVersion::<T>::new();
		let genesis_hash_check = frame_system::CheckGenesis::<T>::new();
		let era_check = frame_system::CheckEra::<T>::from(Era::immortal());

		non_zero_sender_check.pre_dispatch(&who, &runtime_call, info, len)?;
		spec_version_check.pre_dispatch(&who, &runtime_call, info, len)?;
		tx_version_check.pre_dispatch(&who, &runtime_call, info, len)?;
		genesis_hash_check.pre_dispatch(&who, &runtime_call, info, len)?;
		era_check.pre_dispatch(&who, &runtime_call, info, len)
	}

	/// Validates the transaction fee paid with tokens.
	pub fn validate(&self) -> TransactionValidity {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let who = self.0.clone();

		let non_zero_sender_check = frame_system::CheckNonZeroSender::<T>::new();
		let spec_version_check = frame_system::CheckSpecVersion::<T>::new();
		let tx_version_check = frame_system::CheckTxVersion::<T>::new();
		let genesis_hash_check = frame_system::CheckGenesis::<T>::new();
		let era_check = frame_system::CheckEra::<T>::from(Era::immortal());

		let non_zero_sender_validity =
			non_zero_sender_check.validate(&who, &runtime_call, info, len)?;
		let spec_version_validity = spec_version_check.validate(&who, &runtime_call, info, len)?;
		let tx_version_validity = tx_version_check.validate(&who, &runtime_call, info, len)?;
		let genesis_hash_validity = genesis_hash_check.validate(&who, &runtime_call, info, len)?;
		let era_validity = era_check.validate(&who, &runtime_call, info, len)?;

		Ok(non_zero_sender_validity
			.combine_with(spec_version_validity)
			.combine_with(tx_version_validity)
			.combine_with(genesis_hash_validity)
			.combine_with(era_validity))
	}
}
