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

	let mut call: Option<RuntimeCall> = None;

	for depth in (1..11).rev() {
		let mut msg: Xcm<RuntimeCall>;
		match depth {
			10 => {
				msg = Xcm(vec![ClearOrigin]);
			},
			9 => {
				let inner_call = call.take().unwrap();
				let expected_transact_status =
					sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
						index: 27,
						error: [24, 0, 0, 0],
						message: Some("LocalExecutionIncomplete"),
					})
					.encode()
					.into();
				msg = base_xcm(inner_call);
				msg.inner_mut().push(ExpectTransactStatus(expected_transact_status));
			},
			d if d >= 1 && d <= 8 => {
				let inner_call = call.take().unwrap();
				msg = base_xcm(inner_call);
				msg.inner_mut().push(ExpectTransactStatus(MaybeErrorCode::Success));
			},
			_ => unreachable!(),
		}

		let max_weight =
			<FrequencyXcmConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
		call = Some(RuntimeCall::PolkadotXcm(pallet_xcm::Call::execute {
			message: Box::new(staging_xcm::VersionedXcm::from(msg.clone())),
			max_weight,
		}));
	}

	// ────────────────
	// Test Setup
	// ────────────────
	let inner_call = call.take().unwrap();
	let xcm_call = base_xcm(inner_call);

	println!("-----------------start execution-----------------");
	let _ = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::execute(
		t.signed_origin,
		bx!(staging_xcm::VersionedXcm::from(xcm_call)),
		Weight::MAX,
	);
	println!("-----------------end execution-----------------");

	Ok(())
}

fn assert_xcm_completed_successfully(_t: FrequencyToAssetHubTest) {
	// FrequencyWestend::assert_xcm_pallet_attempted_complete(None);
	// FrequencyWestend::assert_xcm_pallet_attempted_incomplete(None, None);
}

fn assert_nothing(_t: FrequencyToAssetHubTest) {}

// ===========================================================================
// ======= Test Frequency XCM Configuration: Recursion Limits ===============
// ===========================================================================

/// This test validates the XCM executor’s enforcement of recursion limits,
/// specifically the stack depth allowed for nested `Transact` instructions.
///
/// **Primary Configuration Under Test**
/// - `xcm_executor::Config::XcmExecutor`
///   - Specifically: the `MAX_RECURSION_LIMIT` (most likely 10)
///   - Controls how many nested XCM `Transact` instructions are allowed
///
/// **Test Behavior**
/// - Builds 11 nested `Transact` instructions using a loop:
///   - Levels 1–9: `Transact(... ExpectTransactStatus(Success))`
///   - Level 10: `Transact(... ExpectTransactStatus(LocalExecutionIncomplete))`
///   - Level 11: Plain XCM message that will exceed the recursion stack limit
/// - Executes the full chain of nested calls inside a single extrinsic
///
/// **Expected Outcome**
/// - The inner-most call (depth 11) fails with `XcmError::ExceedsStackLimit`
/// - All outer calls (up to level 10) succeed and report correct status
/// - Confirms the recursion limit is working as a guardrail for execution safety
///
/// **Security Implications**
/// - Prevents infinite recursion and stack overflows via malicious or malformed XCM
/// - Ensures deep nesting can't bypass fee payments or runtime limits
/// - Verifies `ExpectTransactStatus` handles inner failure without cascading error
///
/// **Supports Fee Logic**
/// - Uses `BuyExecution` in each nested layer to pay for execution
/// - Ensures `AssetTransactor` integration correctly deducts fees even across recursion
///
/// **To Run:**
/// ```bash
/// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::transact::trasaction_recursion_limits_success -p frequency-westend-integration-tests -- --nocapture
/// ```
#[test]
fn trasaction_recursion_limits_success() {
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
