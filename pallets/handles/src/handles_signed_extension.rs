//! Substrate Signed Extension for validating requests to the handles pallet
use crate::{Call, Config, Error, MSAIdToDisplayName};
use common_primitives::msa::MsaValidator;
use core::marker::PhantomData;
use frame_support::{
	dispatch::DispatchInfo, ensure, pallet_prelude::ValidTransaction, traits::IsSubType,
	unsigned::UnknownTransaction,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
	DispatchError, ModuleError,
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

	/// Validates the following criteria for the retire_handle() extrinsic:
	///
	/// * The delegator must already have a MSA id
	/// * The MSA must already have a handle associated with it
	///
	/// Returns a `ValidTransaction` or wrapped `pallet::Error`
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
		let handle_from_state = MSAIdToDisplayName::<T>::try_get(delegator_msa_id)
			.map_err(|_| UnknownTransaction::CannotLookup)?;
		let expiration = handle_from_state.1;
		let current_block = frame_system::Pallet::<T>::block_number();

		let is_past_min_lifetime = current_block >= expiration;
		ensure!(
			is_past_min_lifetime,
			map_dispatch_error(DispatchError::Other(
				Error::<T>::HandleWithinMortalityPeriod.into()
			))
		);

		return ValidTransaction::with_tag_prefix(TAG_PREFIX).build();
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

/// Map a module DispatchError to an InvalidTransaction::Custom error
pub fn map_dispatch_error(err: DispatchError) -> InvalidTransaction {
	InvalidTransaction::Custom(match err {
		DispatchError::Module(ModuleError { error, .. }) => error[0],
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

	/// Additional signed
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	/// Pre dispatch
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
	/// Returns a `ValidTransaction` or wrapped [`TransactionValidityError`]
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
			Some(Call::retire_handle {}) => Self::validate_retire_handle(who),
			_ => Ok(Default::default()),
		}
	}
}
