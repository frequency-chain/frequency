use std::marker::PhantomData;
use codec::{Decode, Encode, };
use frame_support::{traits::IsSubType, weights::DispatchInfo};
use frame_support::traits::EnsureOrigin;
use pallet_democracy::{Call, Config as DemocracyConfig};
use scale_info::{TypeInfo};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, ValidTransaction, TransactionValidity, TransactionValidityError,
	}
};

/// VerifyVoter is a SignedExtension that checks to see if the account casting a vote
/// may vote based on predetermined qualifications, such as being a Major Token
/// Holder that has not fully vested.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VerifyVoter<T: DemocracyConfig, I: 'static>(PhantomData<(T, I)>) ;

/// Errors raised when vote attempt fails validity check
pub enum VoterValidityError {
	/// MTH accounts may not vote
	VotingNotPermitted,
	/// Origin is not permitted to vote for some other reason
	VotingNotPermittedOther,
}
impl<T: DemocracyConfig, I: 'static> VerifyVoter<T, I> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

/// VerifyVoter validation helper functions
impl<T: DemocracyConfig, I: 'static> VerifyVoter<T, I>
// where I: has whatever we want it to have
where I: EnsureOrigin<T::AccountId, Success = T::AccountId>
{

	/// Checks that `account_id` is not on the blocklist
	/// If try_origin succeeds then an error is returned.
	/// If it fails then the transaction may proceed.
	pub fn validate_is_not_blocklisted(&self, account_id: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &'static str = "MTHMembership";

		match <I>::try_origin(account_id) {
			Ok(_) => {
				let err: InvalidTransaction = InvalidTransaction::Custom(VoterValidityError::VotingNotPermitted as u8);
				return TransactionValidity::from(err);
			},
			Err(_) => return  ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id.clone()).build(),
		};
	}
}

/// VerifyVoter Debug trait implementation
impl<T: DemocracyConfig + Send + Sync, I> sp_std::fmt::Debug for VerifyVoter<T, I> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "VerifyVoter")
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// VerifyVoter SignedExtension trait implementation
impl<T: DemocracyConfig + Send + Sync, I: 'static> SignedExtension for VerifyVoter<T, I>
where
	T::Call: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
	I: TypeInfo + Clone + Eq + Send + Sync + EnsureOrigin<T::AccountId, Success=T::AccountId>,
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
			Some(Call::vote { .. }) => Self::validate_is_not_blocklisted(self,who),
			_ => return Ok(Default::default()),
		}
	}
}
