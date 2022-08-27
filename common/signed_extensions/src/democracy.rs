use std::marker::PhantomData;

use codec::{Codec, Decode, Encode, Error, Input};
use frame_support::{traits::IsSubType, weights::DispatchInfo};
use pallet_democracy::{Call, Config};
use scale_info::{StaticTypeInfo, Type, TypeInfo};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
	DispatchError,
};

/// VerifyVoter is a SignedExtension that checks to see if the account casting a vote
/// may vote based on predetermined qualifications, such as being a Major Token
/// Holder that has not fully vested.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VerifyVoter<T: Config + Send + Sync>(PhantomData<T>);

/// Errors raised when vote attempt fails validity check
enum VoterValidityError {
	/// MTH accounts may not vote
	VotingNotPermittedForMTH,
	/// Origin is not permitted to vote for some other reason
	VotingNotPermittedOther,
}

/// VerifyVoter validation helper functions
impl<T: Config + Send + Sync> VerifyVoter<T> {
	/// validate_voter_is_not_mth checks that `account_id` is not a Major Token Holder
	pub fn validate_voter_is_not_mth(_account_id: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &str = "MTHMembership";
		return TransactionValidity::from(InvalidTransaction::Call)
		// return ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id).build();
	}
}

/// VerifyVoter constructor
impl<T: Config + Send + Sync> VerifyVoter<T> {
	/// Create new `VerifyVoter` `SignedExtension` .
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

/// VerifyVoter Debug trait implementation
impl<T: Config + Send + Sync> sp_std::fmt::Debug for VerifyVoter<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "VerifyVoter<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// VerifyVoter SignedExtension trait implementation
impl<T: Config + Send + Sync> SignedExtension for VerifyVoter<T>
where
	T::Call: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::Call;
	type AdditionalSigned = ();
	type Pre = ();

	const IDENTIFIER: &'static str = "VerifyVoter";

	fn additional_signed(&self) -> Result<(), TransactionValidityError> {
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

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		match call.is_sub_type() {
			Some(Call::vote { .. }) => VerifyVoter::<T>::validate_voter_is_not_mth(who),
			_ => return Ok(Default::default()),
		}
	}
}
