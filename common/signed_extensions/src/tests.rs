use crate::{democracy::VerifyVoter, mock::*};

use frame_support::assert_err;

use sp_runtime::traits::SignedExtension;

#[test]
fn signed_extension_validate_voter() {
	new_test_ext().execute_with(|| {
		assert_eq!(true, false);
	})
}
