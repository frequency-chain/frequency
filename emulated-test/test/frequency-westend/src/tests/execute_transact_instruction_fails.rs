use crate::imports::frequency_runtime::{PolkadotXcm, Runtime, RuntimeCall, RuntimeOrigin};
use crate::imports::*;
use frame_support::assert_err_ignore_postinfo;
use parity_scale_codec::Encode;

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
	// ────────────────
	// Test Setup
	// ────────────────
	let xrqcy_transfer_amount = FrequencyExistentialDeposit::get() * 1000;
	let sender = FrequencyWestendSender::get();
	let origin = RuntimeOrigin::signed(sender.clone());
	let remark_call = RuntimeCall::System(frame_system::Call::remark { remark: "test".into() });

	assert_eq!(xrqcy_transfer_amount, 1_000_000_000);

	// ─────────────────────────────
	// Execute + Assert
	// ─────────────────────────────
	FrequencyWestend::execute_with(|| {
		let base_xcm = |call: RuntimeCall| {
			Xcm::<RuntimeCall>(vec![
				WithdrawAsset((Here, xrqcy_transfer_amount).into()),
				BuyExecution {
					fees: (Here, xrqcy_transfer_amount / 2).into(),
					weight_limit: Unlimited,
				},
				Transact {
					origin_kind: OriginKind::Native,
					call: call.encode().into(),
					fallback_max_weight: None,
				},
				RefundSurplus,
			])
		};

		let result = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::execute(
			origin,
			bx!(staging_xcm::VersionedXcm::from(base_xcm(remark_call))),
			Weight::MAX,
		);

		assert_err_ignore_postinfo!(result, pallet_xcm::Error::<Runtime>::LocalExecutionIncomplete);
	});
}
