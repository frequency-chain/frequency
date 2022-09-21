use std::marker::PhantomData;
use codec::{Decode, Encode, };
use frame_support::{traits::IsSubType, weights::DispatchInfo};
use frame_support::traits::EnsureOrigin;
use pallet_democracy::{Call, Config as DemocracyConfig};
use pallet_collective::Config as CollectiveConfig;
use frame_system::Config;
use scale_info::{TypeInfo};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, ValidTransaction, TransactionValidity, TransactionValidityError,
	}
};
// use sp_runtime::traits::IsMember;
//pub trait IsMember<MemberId> {
// 	/// Is the given `MemberId` a valid member?
// 	fn is_member(member_id: &MemberId) -> bool;
// }

pub trait IsMember<MemberId> {
	fn is_member(account_id: &MemberId) -> bool;
}

// have to have CollectiveConfig because that's the only way to define the IsMember trait, because it works
// on an instance that includes an AccountId.
// have to have DemocracyConfig for checking the call
// have to have frame_system::Config for AccountId and Origin?

/// Errors raised when vote attempt fails validity check
pub enum VoterValidityError {
	/// MTH accounts may not vote
	VotingNotPermitted,
	/// Origin is not permitted to vote for some other reason
	VotingNotPermittedOther,
}

/// VerifyVoter is a SignedExtension that checks to see if the account casting a vote
/// may vote based on predetermined qualifications, such as being a Major Token
/// Holder that has not fully vested.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T,I))]
pub struct VerifyVoter<T, I: 'static>(PhantomData<(T, I)>)
	where T: Config + CollectiveConfig<I> + DemocracyConfig;

impl<T, I: 'static> VerifyVoter<T, I>
	where T: Config + CollectiveConfig<I> + DemocracyConfig
{
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

/// VerifyVoter validation helper functions
// where I: EnsureOrigin<T::AccountId, Success = T::AccountId>
impl<T: , I: 'static> VerifyVoter<T, I>
	where T: Config + CollectiveConfig<I> + DemocracyConfig,
	T: CollectiveConfig<I>,
    I: IsMember<T::AccountId>
{
	/// Checks that `account_id` is not on the blocklist
	/// If try_origin succeeds then an error is returned.
	/// If it fails then the transaction may proceed.
	pub fn validate_is_not_blocklisted(&self, account_id: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &'static str = "MTHMembership";
		if <I>::is_member(account_id) {
			let err: InvalidTransaction = InvalidTransaction::Custom(VoterValidityError::VotingNotPermitted as u8);
			return TransactionValidity::from(err);
		}
		ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id.clone()).build()
	}
}


/// VerifyVoter Debug trait implementation
impl<T, I: 'static> sp_std::fmt::Debug for VerifyVoter<T, I>
 where T: CollectiveConfig<I> + DemocracyConfig + Send + Sync,
		I: IsMember<T::AccountId>
{
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
impl<T, I: 'static> SignedExtension for VerifyVoter<T, I>
where
	T: CollectiveConfig<I> + DemocracyConfig + Send + Sync,
	T::Call: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
	I: Clone + Eq + Send + Sync + IsMember<T::AccountId>
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
