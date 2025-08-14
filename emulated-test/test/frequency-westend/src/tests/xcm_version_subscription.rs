use emulated_integration_tests_common::SAFE_XCM_VERSION;

use crate::{imports::*, tests::utils::fr_setup_xcm_version_for_ah};
use staging_xcm::GetVersion;

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
	let starting_xcm_version = 4;

	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());

	// Fund Frequency sovereign account on AH
	AssetHubWestend::fund_para_sovereign(FrequencyWestend::para_id(), amount * 2);

	println!("Setting up AssetHub test environment...");
	let _ = fr_setup_xcm_version_for_ah(starting_xcm_version);
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

	let mut test = FrequencyToAssetHubTest::new(test_args);

	// Read the XCM version SupportedVersion for AssetHub from storage
	FrequencyWestend::execute_with(|| {
		let asset_hub_location = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
		let asset_hub_xcm_version =
			<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::get_version_for(
				&asset_hub_location,
			);
		assert_eq!(asset_hub_xcm_version, Some(starting_xcm_version));
		println!(
			"AssetHub XCM version before subscription: {}",
			asset_hub_xcm_version.unwrap_or(0)
		);
	});

	test.set_assertion::<FrequencyWestend>(assert_subscribe_version_sent);
	test.set_assertion::<AssetHubWestend>(assert_subscribe_version_received_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_asset_hub_subscribe_version);
	test.assert();

	// Read the XCM version SupportedVersion for AssetHub from storage
	FrequencyWestend::execute_with(|| {
		let asset_hub_location = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
		let asset_hub_xcm_version =
			<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::get_version_for(
				&asset_hub_location,
			);
		assert_eq!(asset_hub_xcm_version, Some(SAFE_XCM_VERSION));
		println!("AssetHub XCM version after subscription: {}", asset_hub_xcm_version.unwrap_or(0));
	});
}
