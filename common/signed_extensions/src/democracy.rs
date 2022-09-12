use codec::{ Decode, Encode, };
use frame_support::{traits::IsSubType, weights::DispatchInfo};
use frame_support::traits::EnsureOrigin;
use pallet_democracy::{Call, Config};
use scale_info::{TypeInfo};
use sp_runtime::{traits::{DispatchInfoOf, Dispatchable, SignedExtension}, transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError,
}};
use sp_runtime::transaction_validity::ValidTransaction;

/// VerifyVoter is a SignedExtension that checks to see if the account casting a vote
/// may vote based on predetermined qualifications, such as being a Major Token
/// Holder that has not fully vested.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VerifyVoter<OuterOrigin> {
	member_ensurer: &'static EnsureOrigin<OuterOrigin, Success = OuterOrigin>
}

/// Errors raised when vote attempt fails validity check
pub enum VoterValidityError {
	/// MTH accounts may not vote
	VotingNotPermitted,
	/// Origin is not permitted to vote for some other reason
	VotingNotPermittedOther,
}

/// VerifyVoter validation helper functions
impl<OuterOrigin> VerifyVoter<OuterOrigin> {

	/// Checks that `account_id` is not on the blocklist
	/// If try_origin succeeds then an error is returned.
	/// If it fails then the transaction may proceed.
	pub fn validate_is_not_blocklisted(&self, account_id: OuterOrigin) -> TransactionValidity {
		const TAG_PREFIX: &'static str = "MTHMembership";

		match self.member_ensurer.try_origin(account_id) {
			Ok(_) => {
				let err: InvalidTransaction = InvalidTransaction::Custom(VoterValidityError::VotingNotPermitted as u8);
				return TransactionValidity::from(err);
			},
			Err(_) => return  ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id.clone()).build(),
		};
	}
}

/// VerifyVoter constructor
impl<OuterOrigin> VerifyVoter<OuterOrigin> {
	/// Create new `VerifyVoter` `SignedExtension` .
	pub fn new(member_ensurer: &impl EnsureOrigin<OuterOrigin>) -> Self {
		Self { member_ensurer: member_ensurer.clone() }
	}
}

/// VerifyVoter Debug trait implementation
impl<T: Config + Send + Sync> sp_std::fmt::Debug for VerifyVoter<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "VerifyVoter<{:?}>", self.blacklist)
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
			Some(Call::vote { .. }) => VerifyVoter::<T>::validate_voter_is_not_blacklisted(self,who),
			_ => return Ok(Default::default()),
		}
	}
}
