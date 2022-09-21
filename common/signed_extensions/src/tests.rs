use crate::{democracy::{VerifyVoter, VoterValidityError}, mock::*};

use frame_support::{assert_err, assert_ok};
use frame_support::weights::DispatchInfo;
use pallet_democracy::{AccountVote, Vote, Conviction, ReferendumIndex};

use sp_runtime::{
	AccountId32,
	traits::SignedExtension,
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

const AYE: Vote = Vote { aye: true, conviction: Conviction::None };

pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}



#[test]
fn signed_extension_validate_voter() {
	new_test_ext().execute_with(|| {
		let test_account = test_public(1u8);
		let test_account2 = test_public(2u8);
		let blacklist = vec![test_account.clone()];

		let ref_index: ReferendumIndex = 42;
		let vote =	AccountVote::Standard { vote: AYE, balance: 1 };
		let info = DispatchInfo::default();
		let len = 0_usize;


		let call_vote: &<Test as frame_system::Config>::Call =
			&Call::Democracy(DemocracyCall::vote { ref_index, vote });

		assert_ok!(
			VerifyVoter::<Test, MTHoldersInstance>::new().validate(&test_account2, &call_vote, &info, len)
		);
		// assert_err!(
		// 	VerifyVoter::<Test, DummyOrigin<Test>>::new().validate(&test_account, &call_vote, &info, len),
		// 	TransactionValidityError::Invalid(InvalidTransaction::Custom(VoterValidityError::VotingNotPermitted as u8))
		// )
	})
}

