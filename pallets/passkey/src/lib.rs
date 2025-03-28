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
use common_runtime::{extensions::check_nonce::CheckNonce, signature::check_signature};
use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo, RawOrigin},
	pallet_prelude::*,
	traits::{Contains, OriginTrait},
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::{OnChargeTransaction, Val};
use sp_runtime::{
	generic::Era,
	traits::{
		AsTransactionAuthorizedOrigin, Convert, Dispatchable, SignedExtension, TxBaseImplication,
		Zero,
	},
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
#[cfg(test)]
mod tests_v2;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::tokens::fungible::Mutate;
use frame_system::CheckWeight;
use sp_runtime::traits::{DispatchTransaction, TransactionExtension, ValidateResult};

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

		/// Helper Currency method for benchmarking
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
		/// Proxies an extrinsic call by changing the origin to `account_id` inside the payload.
		/// Since this is an unsigned extrinsic all the verification checks are performed inside
		/// `validate_unsigned` and `pre_dispatch` hooks.
		#[deprecated(since = "1.15.2", note = "Use proxy_v2 instead")]
		#[pallet::call_index(0)]
		#[pallet::weight({
			let dispatch_info = payload.passkey_call.call.get_dispatch_info();
			let overhead = <T as Config>::WeightInfo::pre_dispatch();
			let total = overhead.saturating_add(dispatch_info.call_weight);
			(total, dispatch_info.class)
		})]
		#[allow(deprecated)]
		pub fn proxy(
			origin: OriginFor<T>,
			payload: PasskeyPayload<T>,
		) -> DispatchResultWithPostInfo {
			Self::proxy_v2(origin, payload.into())
		}

		/// Proxies an extrinsic call by changing the origin to `account_id` inside the payload.
		/// Since this is an unsigned extrinsic all the verification checks are performed inside
		/// `validate_unsigned` and `pre_dispatch` hooks.
		#[pallet::call_index(1)]
		#[pallet::weight({
			let dispatch_info = payload.passkey_call.call.get_dispatch_info();
			let overhead = <T as Config>::WeightInfo::pre_dispatch();
			let total = overhead.saturating_add(dispatch_info.call_weight);
			(total, dispatch_info.class)
		})]
		pub fn proxy_v2(
			origin: OriginFor<T>,
			payload: PasskeyPayloadV2<T>,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let transaction_account_id = payload.passkey_call.account_id.clone();
			let main_origin = T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(
				transaction_account_id.clone(),
			));
			let result = payload.passkey_call.call.dispatch(main_origin);
			if let Ok(_inner) = result {
				// all post-dispatch logic should be included in here
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
		<T as frame_system::Config>::RuntimeOrigin: AsTransactionAuthorizedOrigin,
	{
		type Call = Call<T>;

		/// Validating the regular checks of an extrinsic plus verifying the P256 Passkey signature
		/// The majority of these checks are the same as `SignedExtra` list in defined in runtime
		/// TODO: pass source down....
		/// TODO: in the new system it looks like you are not meant to call other, non-unsigned
		/// validations outside your pallet. Other validations now require additional parameters
		/// but most extensions will not use them:
		/// 	_self_implicit: Self::Implicit,
		// 		_inherited_implication: &impl Encode,
		// 		_source: TransactionSource,
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = ValidTransaction::default();
			let (payload, is_legacy_call) = Self::filter_valid_calls(&call)?;

			let frame_system_validity =
				FrameSystemChecks(payload.passkey_call.account_id.clone(), call.clone())
					.validate()?;
			let nonce_validity = PasskeyNonceCheck::new(payload.passkey_call.clone()).validate()?;
			let weight_validity =
				PasskeyWeightCheck::new(payload.passkey_call.account_id.clone(), call.clone())
					.validate()?;
			let tx_payment_validity = ChargeTransactionPayment::<T>(
				payload.passkey_call.account_id.clone(),
				call.clone(),
			)
			.validate()?;
			// this is the last since it is the heaviest
			let signature_validity =
				PasskeySignatureCheck::new(payload.clone(), is_legacy_call).validate()?;

			let valid_tx = valid_tx
				.combine_with(frame_system_validity)
				.combine_with(nonce_validity)
				.combine_with(weight_validity)
				.combine_with(tx_payment_validity)
				.combine_with(signature_validity);
			Ok(valid_tx)
		}

		/// Checking and executing a list of operations pre_dispatch
		/// The majority of these checks are the same as `SignedExtra` list in defined in runtime
		fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
			let (payload, is_legacy_call) = Self::filter_valid_calls(&call)?;
			FrameSystemChecks(payload.passkey_call.account_id.clone(), call.clone())
				.pre_dispatch()?;
			PasskeyNonceCheck::new(payload.passkey_call.clone()).pre_dispatch()?;
			PasskeyWeightCheck::new(payload.passkey_call.account_id.clone(), call.clone())
				.pre_dispatch()?;

			ChargeTransactionPayment::<T>(payload.passkey_call.account_id.clone(), call.clone())
				.pre_dispatch()?;
			// this is the last since it is the heaviest
			PasskeySignatureCheck::new(payload.clone(), is_legacy_call).pre_dispatch()
		}
	}
}

impl<T: Config> Pallet<T>
where
	BalanceOf<T>: Send + Sync + From<u64>,
	<T as frame_system::Config>::RuntimeCall:
		From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	/// Filtering the valid calls and extracting the Payload V2 from inside the call and returning if this
	/// is a legacy call or not
	fn filter_valid_calls(
		call: &Call<T>,
	) -> Result<(PasskeyPayloadV2<T>, bool), TransactionValidityError> {
		match call {
			Call::proxy { payload }
				if T::PasskeyCallFilter::contains(&payload.clone().passkey_call.call) =>
				Ok((payload.clone().into(), true)),
			Call::proxy_v2 { payload }
				if T::PasskeyCallFilter::contains(&payload.clone().passkey_call.call) =>
				Ok((payload.clone(), false)),
			_ => Err(InvalidTransaction::Call.into()),
		}
	}
}

/// Passkey specific nonce check which is a wrapper around `CheckNonce` extension
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
struct PasskeyNonceCheck<T: Config>(pub PasskeyCallV2<T>);

impl<T: Config> PasskeyNonceCheck<T>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo>,
{
	pub fn new(passkey_call: PasskeyCallV2<T>) -> Self {
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

/// Passkey signatures check which verifies 2 signatures
/// 1. Account signature of the P256 public key
/// 2. Passkey P256 signature of the account public key
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
struct PasskeySignatureCheck<T: Config> {
	payload: PasskeyPayloadV2<T>,
	is_legacy_payload: bool,
}

impl<T: Config> PasskeySignatureCheck<T> {
	pub fn new(passkey_payload: PasskeyPayloadV2<T>, is_legacy_payload: bool) -> Self {
		Self { payload: passkey_payload, is_legacy_payload }
	}

	pub fn validate(&self) -> TransactionValidity {
		// checking account signature to verify ownership of the account used
		let signed_data = self.payload.passkey_public_key.clone();
		let signature = self.payload.account_ownership_proof.clone();
		let signer = &self.payload.passkey_call.account_id;

		Self::check_account_signature(signer, &signed_data.inner().to_vec(), &signature)
			.map_err(|_e| TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

		// checking the passkey signature to ensure access to the passkey
		let p256_signed_data = match self.is_legacy_payload {
			true => PasskeyPayload::from(self.payload.clone()).passkey_call.encode(),
			false => self.payload.passkey_call.encode(),
		};
		let p256_signature = self.payload.verifiable_passkey_signature.clone();
		let p256_signer = self.payload.passkey_public_key.clone();

		p256_signature
			.try_verify(&p256_signed_data, &p256_signer)
			.map_err(|e| match e {
				PasskeyVerificationError::InvalidProof =>
					TransactionValidityError::Invalid(InvalidTransaction::BadSigner),
				_ => TransactionValidityError::Invalid(InvalidTransaction::Custom(e.into())),
			})?;

		Ok(ValidTransaction::default())
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

		if !check_signature(signature, key, signed_data.clone()) {
			return Err(Error::<T>::InvalidAccountSignature.into());
		}

		Ok(())
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
	<T as frame_system::Config>::RuntimeOrigin: AsTransactionAuthorizedOrigin,
{
	/// Validates the transaction fee paid with tokens.
	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());

		let raw_origin = RawOrigin::from(Some(self.0.clone()));
		let who = <T as frame_system::Config>::RuntimeOrigin::from(raw_origin);
		pallet_transaction_payment::ChargeTransactionPayment::<T>::from(Zero::zero())
			.validate_and_prepare(who, &runtime_call, info, len, 4)?;
		Ok(())
	}

	/// Validates the transaction fee paid with tokens.
	pub fn validate(&self) -> TransactionValidity {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let raw_origin = RawOrigin::from(Some(self.0.clone()));
		let who = <T as frame_system::Config>::RuntimeOrigin::from(raw_origin);

		pallet_transaction_payment::ChargeTransactionPayment::<T>::from(Zero::zero()).validate(
			who,
			&runtime_call,
			info,
			len,
			(),
			&TxBaseImplication(runtime_call.clone()), // implication
			TransactionSource::External,
		)?;
		Ok(ValidTransaction::default())
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

		let raw_origin = RawOrigin::from(Some(self.0.clone()));
		let who = <T as frame_system::Config>::RuntimeOrigin::from(raw_origin);

		let non_zero_sender_check = frame_system::CheckNonZeroSender::<T>::new();
		let spec_version_check = frame_system::CheckSpecVersion::<T>::new();
		let tx_version_check = frame_system::CheckTxVersion::<T>::new();
		let genesis_hash_check = frame_system::CheckGenesis::<T>::new();
		let era_check = frame_system::CheckEra::<T>::from(Era::immortal());

		// currently (in stable2412) these implement the default version of `prepare`, which always returns Ok(...)
		// val, origin, call, info, len (?)
		non_zero_sender_check.prepare((), &who, &runtime_call, info, len)?;
		spec_version_check.prepare((), &who, &runtime_call, info, len)?;
		tx_version_check.prepare((), &who, &runtime_call, info, len)?;
		genesis_hash_check.prepare((), &who, &runtime_call, info, len)?;
		era_check.prepare((), &who, &runtime_call, info, len)
	}

	/// Validates the transaction fee paid with tokens.
	pub fn validate(&self) -> TransactionValidity {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let implication = TxBaseImplication(runtime_call.clone());
		let raw_origin = RawOrigin::from(Some(self.0.clone()));
		let who = <T as frame_system::Config>::RuntimeOrigin::from(raw_origin);

		let non_zero_sender_check = frame_system::CheckNonZeroSender::<T>::new();
		let spec_version_check = frame_system::CheckSpecVersion::<T>::new();
		let tx_version_check = frame_system::CheckTxVersion::<T>::new();
		let genesis_hash_check = frame_system::CheckGenesis::<T>::new();
		let era_check = frame_system::CheckEra::<T>::from(Era::immortal());

		// FIXME: give these the right params
		let (non_zero_sender_validity, _, origin) = non_zero_sender_check.validate(
			who.clone(),
			&runtime_call,
			info,
			len,
			(),
			&implication,
			TransactionSource::External,
		)?;

		let (spec_version_validity, _, origin) = spec_version_check.validate(
			origin,
			&runtime_call,
			info,
			len,
			spec_version_check.implicit().unwrap(),
			&implication,
			TransactionSource::External,
		)?; // TODO: unwrap

		let (tx_version_validity, _, origin) = tx_version_check.validate(
			origin,
			&runtime_call,
			info,
			len,
			tx_version_check.implicit().unwrap(),
			&implication,
			TransactionSource::External,
		)?; // TODO: unwrap

		let (genesis_hash_validity, _, origin) = genesis_hash_check.validate(
			origin,
			&runtime_call,
			info,
			len,
			genesis_hash_check.implicit().unwrap(),
			&implication,
			TransactionSource::External,
		)?; // TODO: unwrap

		let (era_validity, _, _) = era_check.validate(
			origin,
			&runtime_call,
			info,
			len,
			era_check.implicit().unwrap(),
			&implication,
			TransactionSource::External,
		)?; // TODO: unwrap

		Ok(non_zero_sender_validity
			.combine_with(spec_version_validity)
			.combine_with(tx_version_validity)
			.combine_with(genesis_hash_validity)
			.combine_with(era_validity))
	}
}

/// Block resource (weight) limit check.
#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyWeightCheck<T: Config>(pub T::AccountId, pub Call<T>);

impl<T: Config> PasskeyWeightCheck<T>
where
	<T as frame_system::Config>::RuntimeCall:
		From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	/// creating a new instance
	pub fn new(account_id: T::AccountId, call: Call<T>) -> Self {
		Self(account_id, call)
	}

	/// Validate the transaction
	pub fn validate(&self) -> TransactionValidity {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());
		let implication = TxBaseImplication(runtime_call.clone());
		let raw_origin = RawOrigin::from(Some(self.0.clone()));
		let who = <T as frame_system::Config>::RuntimeOrigin::from(raw_origin);

		let check_weight = CheckWeight::<T>::new();
		let (result, _, _) = check_weight.validate(
			who,
			&runtime_call,
			&info,
			len,
			(),
			&implication,
			TransactionSource::External,
		)?;
		Ok(result)
	}

	/// Pre-dispatch transaction checks
	/// FIXME:  prepare expects a first parameter = the next length of data in the block.
	// This is how it's done internally in check_weight, and would work if `check_block_length`
	// weren't private:
	// let next_len = CheckWeight::<T>::check_block_length(info, len)?;
	pub fn pre_dispatch(&self) -> Result<(), TransactionValidityError> {
		let info = &self.1.get_dispatch_info();
		let len = self.1.using_encoded(|c| c.len());
		let runtime_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::from(self.1.clone());

		CheckWeight::<T>::bare_validate_and_prepare(&runtime_call, info, len)
	}
}
