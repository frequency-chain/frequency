//! Substrate Signed Extension for validating requests to the handles pallet
use crate::{Call, Config, Error, MSAIdToDisplayName};
use common_primitives::msa::MsaValidator;
use core::marker::PhantomData;
use frame_support::{
	dispatch::DispatchInfo, ensure, pallet_prelude::ValidTransaction, traits::IsSubType,
	unsigned::UnknownTransaction,
};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AsSystemOriginSigner, DispatchInfoOf, Dispatchable, PostDispatchInfoOf,
		TransactionExtension,
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	DispatchError, DispatchResult, ModuleError, Weight,
};

/// The SignedExtension trait is implemented on CheckFreeExtrinsicUse to validate the request. The
/// purpose of this is to ensure that the retire_handle extrinsic cannot be
/// repeatedly called to flood the network.
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct HandlesSignedExtension<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> HandlesSignedExtension<T> {
	/// Create new `SignedExtension`.
	pub fn new() -> Self {
		Self(core::marker::PhantomData)
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
		let delegator_msa_id =
			T::MsaInfoProvider::ensure_valid_msa_key(delegator_key).map_err(map_dispatch_error)?;
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

		ValidTransaction::with_tag_prefix(TAG_PREFIX).build()
	}
}

impl<T: Config + Send + Sync> core::fmt::Debug for HandlesSignedExtension<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "HandlesSignedExtension<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
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

/// The info passed between the validate and prepare steps for the `HandlesSignedExtension` extension.
pub enum Val {
	/// Valid transaction, no weight refund.
	Valid,
	/// Weight refund for the transaction.
	Refund(Weight),
}

/// The info passed between the prepare and post-dispatch steps for the `HandlesSignedExtension` extension.
pub enum Pre {
	/// Valid transaction, no weight refund.
	Valid,
	/// Weight refund for the transaction.
	Refund(Weight),
}

impl<T: Config + Send + Sync> TransactionExtension<T::RuntimeCall> for HandlesSignedExtension<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
	<T as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
{
	const IDENTIFIER: &'static str = "HandlesSignedExtension";
	type Implicit = ();
	type Val = Val;
	type Pre = Pre;

	fn weight(&self, _call: &T::RuntimeCall) -> Weight {
		Weight::zero()
	}
	/// Frequently called by the transaction queue to validate all free Handles extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`TransactionValidityError`]
	/// * retire_handle
	///
	/// Validate functions for the above MUST prevent errors in the extrinsic logic to prevent spam.
	///
	/// Arguments:
	/// who: AccountId calling the extrinsic
	/// call: The pallet extrinsic being called
	/// unused: _info, _len
	///
	fn validate(
		&self,
		origin: <T as frame_system::Config>::RuntimeOrigin,
		call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl parity_scale_codec::Encode,
		_source: TransactionSource,
	) -> sp_runtime::traits::ValidateResult<Self::Val, T::RuntimeCall> {
		let Some(who) = origin.as_system_origin_signer() else {
			return Ok((ValidTransaction::default(), Val::Refund(self.weight(call)), origin));
		};
		let validity = match call.is_sub_type() {
			Some(Call::retire_handle {}) => Self::validate_retire_handle(&who),
			_ => Ok(Default::default()),
		};
		validity.map(|v| (v, Val::Valid, origin))
	}

	fn prepare(
		self,
		val: Self::Val,
		_origin: &<T as frame_system::Config>::RuntimeOrigin,
		_call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		match val {
			Val::Valid => Ok(Pre::Valid),
			Val::Refund(w) => Ok(Pre::Refund(w)),
		}
	}

	fn post_dispatch_details(
		pre: Self::Pre,
		_info: &DispatchInfo,
		_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		_len: usize,
		_result: &DispatchResult,
	) -> Result<Weight, sp_runtime::transaction_validity::TransactionValidityError> {
		match pre {
			Pre::Valid => Ok(Weight::zero()),
			Pre::Refund(w) => Ok(w),
		}
	}
}
