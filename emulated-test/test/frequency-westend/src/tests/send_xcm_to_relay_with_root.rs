use crate::{foreign_balance_on, imports::*, tests::utils::fr_setup_xcm_version_for_westend};

fn frequency_to_relay_send_xcm(t: FrequencyToRelayTest) -> DispatchResult {
	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;
	let asset_on_relay: Assets = (Here, 1_000_000_000_000u128).into();
	let asset_fees: Assets = (Here, 1_000_000_000_000u128 / 2).into();

	let remote_xcm = Xcm::<()>(vec![
		WithdrawAsset(asset_on_relay.clone()),
		BuyExecution { fees: asset_fees.get(0).unwrap().clone(), weight_limit: Unlimited },
		DepositAsset {
			assets: Wild(All),
			beneficiary: AccountId32Junction { network: None, id: WestendReceiver::get().into() }
				.into(),
		},
	]);

	let sudo_inner_call: <FrequencyWestend as Chain>::RuntimeCall =
		RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
			dest: bx!(VersionedLocation::V5(Parent.into())),
			message: bx!(VersionedXcm::V5(remote_xcm)),
		});

	let _ = <FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(
		t.root_origin,
		bx!(sudo_inner_call),
	);

	Ok(())
}

fn assert_xcm_processed_on_relay(_t: FrequencyToRelayTest) {
	Westend::assert_ump_queue_processed(
		true,
		Some(FrequencyWestend::para_id()),
		Some(Weight::from_parts(311_685_000, 7_186)),
	);
}

fn assert_xcm_is_sent_to_relay(_t: FrequencyToRelayTest) {
	FrequencyWestend::assert_parachain_system_ump_sent();
}

pub fn fund_sov_frequency_on_westend(amount: Balance) {
	let freq_location = Westend::child_location_of(FrequencyWestend::para_id());
	let sov_account = Westend::sovereign_account_id_of(freq_location);
	Westend::fund_accounts(vec![(sov_account.into(), amount)]);
}

// =========================================================================
// ========= Send XCM to Relay ===========
// =========================================================================
// This test is used to test the XCM sending functionality from Frequency to Relay.
// It test that the fee is waived for the XCM if it comes from root.
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::send_xcm_to_relay_with_root -- --nocapture
/// Tests reserve transfer of DOT from Frequency to Relay (Westend) with root.
/// This test passing checks that the FeeManager is setup to waive the delivery fee for XCMs sent from root.
#[test]
fn send_xcm_to_relay_with_root() {
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;
	let starting_xcm_version = 5;

	fr_setup_xcm_version_for_westend(starting_xcm_version)
		.expect("Failed to set up XCM version for Westend");
	fund_sov_frequency_on_westend(amount_to_send * 2);

	let assets: Assets = (Parent, amount_to_send).into();
	let destination = FrequencyWestend::parent_location();

	let freq_location = Westend::child_location_of(FrequencyWestend::para_id());
	let sov_account = Westend::sovereign_account_id_of(freq_location);
	assert_eq!(relay_balance_of(&sov_account), 20_000_000_000_000);

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver,
			amount_to_send,
			assets.clone(),
			None,
			0,
		),
	};

	let mut test = FrequencyToRelayTest::new(test_args);

	let dot_location = WestendLocation::get();

	test.set_assertion::<FrequencyWestend>(assert_xcm_is_sent_to_relay);
	test.set_assertion::<Westend>(assert_xcm_processed_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_send_xcm);
	test.assert();

	// Checks that the treasury account has not been credited since the deliveryfee is waived.
	let treasury_account_balance = foreign_balance_on!(
		FrequencyWestend,
		dot_location.clone(),
		&FrequencyTreasuryAccount::get()
	);
	assert_eq!(treasury_account_balance, 0u128, "Treasury account should NOT have been credited");
}
