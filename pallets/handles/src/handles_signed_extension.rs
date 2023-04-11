//! Substrate Signed Extension for validating requests to the handles pallet
use crate::{Call, Config, MSAIdToDisplayName, Pallet};
use codec::{Decode, Encode};
use common_primitives::msa::MsaValidator;
use core::marker::PhantomData;
use frame_support::{
	dispatch::{DispatchInfo, Dispatchable},
	pallet_prelude::ValidTransaction,
	traits::IsSubType,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, SignedExtension},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
	DispatchError,
};
/// The SignedExtension trait is implemented on CheckFreeExtrinsicUse to validate the request. The
/// purpose of this is to ensure that the retire_handle extrinsic cannot be
/// repeatedly called to flood the network.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct HandlesSignedExtension<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> HandlesSignedExtension<T> {
	/// Create new `SignedExtension`.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for HandlesSignedExtension<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "HandlesSignedExtension<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// Validator trait to be used in validating Handles requests from a Signed Extension.
pub trait HandlesValidate<C: Config> {
	/// Validates calls to the retire_handle() extrinsic
	fn validate_retire_handle(delegator_key: &C::AccountId) -> TransactionValidity;
}

impl<T: Config> HandlesValidate<T> for Pallet<T> {
	/// Validates the following criteria for the retire_handle() extrinsic:
	///
	/// * The delegator must already have a MSA id
	/// * The MSA must already have a handle associated with it
	///
	/// Returns a `ValidTransaction` or wrapped [`pallet::Error`]
	///
	/// # Errors (as u8 wrapped by `InvalidTransaction::Custom`)
	/// * [`Error::InvalidMessageSourceAccount`]
	/// * [`Error::MSAHandleDoesNotExist`]
	fn validate_retire_handle(delegator_key: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &str = "HandlesRetireHandle";

		// Validation: The delegator must already have a MSA id
		let delegator_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
			.map_err(|e| map_dispatch_error(e))?;
		// Validation: The MSA must already have a handle associated with it
		_ = MSAIdToDisplayName::<T>::try_get(delegator_msa_id)
			.map_err(|_| map_dispatch_error(DispatchError::CannotLookup))?;
		return ValidTransaction::with_tag_prefix(TAG_PREFIX).build()
	}
}

/// Map a module DispatchError to an InvalidTransaction::Custom error
pub fn map_dispatch_error(err: DispatchError) -> InvalidTransaction {
	InvalidTransaction::Custom(match err {
		DispatchError::Module(module_err) =>
			<u32 as Decode>::decode(&mut module_err.error.as_slice())
				.unwrap_or_default()
				.try_into()
				.unwrap_or_default(),
		_ => 255u8,
	})
}

impl<T: Config + Send + Sync> SignedExtension for HandlesSignedExtension<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "HandlesSignedExtension";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		self.validate(who, call, info, len).map(|_| ())
	}

	/// Frequently called by the transaction queue to validate all free Handles extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`ValidityError`]
	/// * retire_handle
	/// Validate functions for the above MUST prevent errors in the extrinsic logic to prevent spam.
	///
	/// Arguments:
	/// who: AccountId calling the extrinsic
	/// call: The pallet extrinsic being called
	/// unused: _info, _len
	///
	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		match call.is_sub_type() {
			Some(Call::retire_handle {}) => Pallet::<T>::validate_retire_handle(who),
			_ => Ok(Default::default()),
		}
	}
}
