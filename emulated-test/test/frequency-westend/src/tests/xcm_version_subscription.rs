use emulated_integration_tests_common::SAFE_XCM_VERSION;

use crate::{imports::*, tests::utils::setup_xcm_version_for_fr_ah};

// =========================================================================
// ================ XCM Version Subscription Tests ========================
// =========================================================================

fn frequency_to_asset_hub_subscribe_version(t: FrequencyToAssetHubTest) -> DispatchResult {
	// Call the force_default_xcm_version function
	let set_default_xcm_op =
		<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_default_xcm_version(
			t.root_origin.clone(),
			Some(SAFE_XCM_VERSION),
		);
	assert_ok!(set_default_xcm_op);

	let force_subscribe_version_op =
		<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_subscribe_version_notify(
			t.root_origin.clone(),
			bx!(VersionedLocation::V5(FrequencyWestend::sibling_location_of(
				AssetHubWestend::para_id()
			))),
		);
	assert_ok!(force_subscribe_version_op);

	Ok(())
}

fn assert_subscribe_version_sent(_t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	// Check that the SubscribeVersion notification request was emitted
	assert_expected_events!(
		FrequencyWestend,
		vec![
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::VersionNotifyRequested { destination, .. }) => {
				// Expect destination to be the sibling (AssetHub) para location we requested, not Parent
				destination: *destination == FrequencyWestend::sibling_location_of(AssetHubWestend::para_id()),
			},
		]
	);
}

fn assert_subscribe_version_received_on_relay(_t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;

	// With BuyExecution, the message should be processed successfully
	// Look for successful message processing events
	assert_expected_events!(
		AssetHubWestend,
		vec![
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

#[test]
fn test_subscribe_version() {
	let sender = FrequencyWestendSender::get();
	// let receiver = WestendReceiver::get();
	let receiver = AssetHubWestendReceiver::get();
	let amount = 1_000_000_000_000;

	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());

	// Fund Frequency sovereign account on AH
	AssetHubWestend::fund_para_sovereign(FrequencyWestend::para_id(), amount * 2);

	println!("Setting up AssetHub test environment...");
	let _ = setup_xcm_version_for_fr_ah();
	println!("AssetHub test environment set up successfully.");

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver,
			amount,                  // Use non-zero amount
			(Parent, amount).into(), // Non-empty assets
			None,
			0,
		),
	};
	
	FrequencyWestend::execute_with(|| {
		let result = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::get_version_for(&destination);
		println!("before before");
		println!("--------------result: {:?}", result);
	});

	let mut test = FrequencyToAssetHubTest::new(test_args);

	// Re-enable assertion once pattern fixed
	// test.set_assertion::<FrequencyWestend>(assert_subscribe_version_sent);
	// test.set_assertion::<AssetHubWestend>(assert_subscribe_version_received_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_asset_hub_subscribe_version);
	test.assert();

	// let a = pallet_xcm::SupportedVersion::<<FrequencyWestend as Chain>::Runtime>>::get(SAFE_XCM_VERSION, pallet_xcm::LatestVersionedLocation(&destination.clone()));

	// <FrequencyWestend as Chain>::Runtime::get_version_for(&destination);
	FrequencyWestend::execute_with(|| {
		let result = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::get_version_for(&destination);
		println!("taco taco");
		println!("--------------result: {:?}", result);
	});
}
