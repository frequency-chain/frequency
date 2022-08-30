use crate::{democracy::VerifyVoter, mock::*};

use frame_support::assert_ok;
use frame_support::weights::DispatchInfo;
use pallet_democracy::{AccountVote, Vote, Conviction, ReferendumIndex};

use sp_runtime::traits::SignedExtension;
const AYE: Vote = Vote { aye: true, conviction: Conviction::None };

#[test]
fn signed_extension_validate_voter() {
	new_test_ext().execute_with(|| {
		let test_account: AccountId = 1;
		let ref_index: ReferendumIndex = 42;
		let vote =	AccountVote::Standard { vote: AYE, balance: 1 };
		let info = DispatchInfo::default();
		let len = 0_usize;


		let call_vote: &<Test as frame_system::Config>::Call =
			&Call::Democracy(DemocracyCall::vote { ref_index, vote });
		let info = DispatchInfo::default();
		let result = VerifyVoter::<Test>::new().validate(
			&test_account, &call_vote, &info, len,
		);
		assert_ok!(result);
	})
}

#[test]
fn verify_voter_fails_when_mth() {
	new_test_ext().execute_with(|_| {

	})
}
