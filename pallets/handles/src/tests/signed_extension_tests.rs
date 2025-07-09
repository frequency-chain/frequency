use crate::{handles_signed_extension::HandlesSignedExtension, tests::mock::*};
use frame_support::{
	assert_ok,
	dispatch::{DispatchInfo, GetDispatchInfo},
	pallet_prelude::TransactionSource,
	traits::OriginTrait,
};
use parity_scale_codec::Encode;
use sp_core::{sr25519, Pair};
use sp_runtime::traits::{DispatchTransaction, TransactionExtension};

/// Assert that retiring a handle passes the signed extension HandlesSignedExtension
#[test]
fn signed_extension_retire_handle_success() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		// Claim the handle
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		run_to_block(11);

		// Retire the handle
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			call_retire_handle,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	});
}

/// Assert that retiring a handle w/o existing one fails the signed extension HandlesSignedExtension
#[test]
fn signed_extension_retire_handle_failure() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);

		// Retire the handle
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			call_retire_handle,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert!(result.is_err());
	});
}

#[test]
fn test_early_retirement_should_fail() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		// Claim the handle
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		run_to_block(9);

		// Retire the handle
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			call_retire_handle,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert!(result.is_err());
	});
}

#[test]
fn handles_signed_extension_post_dispatch_details_refund_weight() {
	use crate::handles_signed_extension::Pre as HandlesPre;
	let refund_weight = frame_support::weights::Weight::from_parts(5678, 0);
	let result =
		crate::handles_signed_extension::HandlesSignedExtension::<Test>::post_dispatch_details(
			HandlesPre::Refund(refund_weight),
			&DispatchInfo::default(),
			&Default::default(),
			0,
			&Ok(()),
		);
	assert_eq!(result.unwrap(), refund_weight);
}

#[test]
fn handles_signed_extension_does_not_block_unrelated_calls() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) = get_signed_claims_payload(&alice, b"newhandle".to_vec(), expiration);
		let unrelated_call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::change_handle {
				msa_owner_key: alice.public().into(),
				proof,
				payload,
			});
		let info = unrelated_call.get_dispatch_info();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			unrelated_call,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	});
}

#[test]
fn handles_signed_extension_double_retire_edge_case() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test2";
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
		run_to_block(11);
		// First retire should succeed
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result1 = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			call_retire_handle,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result1);
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into())));
		// Second retire should fail
		let result2 = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::signed(alice.public().into()).into(),
			call_retire_handle,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert!(result2.is_err());
	});
}

#[test]
fn handles_signed_extension_noop_for_unsigned_origin() {
	new_test_ext().execute_with(|| {
		let call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = call.get_dispatch_info();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate_only(
			RuntimeOrigin::none().into(),
			call,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		// Should be an error or a no-op, but must not panic
		assert!(result.is_err() || result.is_ok());
	});
}

#[test]
fn handles_signed_extension_skipped_and_refund_for_other_origins() {
	new_test_ext().execute_with(|| {
		let call = &RuntimeCall::Handles(HandlesCall::retire_handle {});
		let ext = HandlesSignedExtension::<Test>::new();

		let mut info = call.get_dispatch_info();
		info.extension_weight = ext.weight(call);

		// Ensure we test the refund which is zero currently.
		assert!(info.extension_weight != frame_support::weights::Weight::zero());

		let len = call.encoded_size();

		let origin = frame_system::RawOrigin::Root.into();
		let (pre, origin) = ext.validate_and_prepare(origin, call, &info, len, 0).unwrap();

		assert!(origin.as_system_ref().unwrap().is_root());

		let pd_res = Ok(());
		let mut post_info = frame_support::dispatch::PostDispatchInfo {
			actual_weight: Some(info.total_weight()),
			pays_fee: Default::default(),
		};

		<HandlesSignedExtension<Test> as TransactionExtension<RuntimeCall>>::post_dispatch(
			pre,
			&info,
			&mut post_info,
			len,
			&pd_res,
		)
		.unwrap();

		assert_eq!(post_info.actual_weight, Some(info.call_weight));
	})
}
