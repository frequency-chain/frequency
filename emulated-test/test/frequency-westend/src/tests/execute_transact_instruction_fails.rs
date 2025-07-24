use crate::imports::*;

use parity_scale_codec::Encode;
use xcm_executor::traits::WeightBounds;

fn build_assets(native_token: Balance) -> Vec<Asset> {
	vec![(Here, native_token).into()]
}

fn execute_xcm_frequency(t: FrequencyToAssetHubTest) -> DispatchResult {
	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;

	let base_xcm = |call: RuntimeCall| {
		Xcm::<RuntimeCall>(vec![
			WithdrawAsset((Here, 1_000).into()),
			BuyExecution { fees: (Here, 1).into(), weight_limit: Unlimited },
			Transact {
				origin_kind: OriginKind::Native,
				call: call.encode().into(),
				fallback_max_weight: None,
			},
		])
	};

	// ────────────────
	// Test Setup
	// ────────────────
	println!("-----------------start execution-----------------");
	let remark_call = RuntimeCall::System(frame_system::Call::remark { remark: "test".into() });
	let result = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::execute(
		t.signed_origin,
		bx!(staging_xcm::VersionedXcm::from(base_xcm(remark_call))),
		Weight::MAX,
	);
	println!("-----------------end execution-----------------");
	// assert_err!(result, XcmError::NoPermission);

	println!("-----------------result----------------- {:?}", result);

	Ok(())
}

fn assert_xcm_completed_successfully(_t: FrequencyToAssetHubTest) {
	FrequencyWestend::assert_xcm_pallet_attempted_complete(None);
	// FrequencyWestend::assert_xcm_pallet_attempted_incomplete(None, None);
	FrequencyWestend::assert_xcm_pallet_attempted_error(Some(XcmError::NoPermission));
}

fn assert_nothing(_t: FrequencyToAssetHubTest) {}

// ===========================================================================
// ======= Test XCM SafeCallFilter Configuration: Dissallow Transact Instruction ===============
// ===========================================================================

/// This test validates the XCM executor’s enforcement of recursion limits,
/// specifically the stack depth allowed for nested `Transact` instructions.
/// **To Run:**
/// ```bash
/// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::execute_transact_instruction_fails -- --nocapture
/// ```
#[test]
fn execute_transact_instruction_fails() {
	let xrqcy_transfer_amount = FrequencyExistentialDeposit::get() * 1000;

	let sender = FrequencyWestendSender::get();
	let receiver = AssetHubWestendReceiver::get();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let assets: Assets = build_assets(xrqcy_transfer_amount).into();

	// ─────────────────────────────
	// Build Test Context
	// ─────────────────────────────
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(destination, receiver, 0u128, assets, None, 0u32),
	};

	let mut test = FrequencyToAssetHubTest::new(test_args);

	// ─────────────────────────────
	// Execute + Assert
	// ─────────────────────────────
	test.set_assertion::<FrequencyWestend>(assert_xcm_completed_successfully);
	test.set_assertion::<AssetHubWestend>(assert_nothing);
	test.set_dispatchable::<FrequencyWestend>(execute_xcm_frequency);
	test.assert();
}
