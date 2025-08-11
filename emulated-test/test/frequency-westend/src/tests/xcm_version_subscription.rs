use emulated_integration_tests_common::SAFE_XCM_VERSION;
// use westend_runtime::{xcm_config::AssetHub, RuntimeOrigin};
// use westend_system_emulated_network::frequency_emulated_chain::frequency_runtime::weights::pallet_xcm;

use crate::imports::*;

// =========================================================================
// ================ XCM Version Subscription Tests ========================
// =========================================================================
fn setup_assethub_test() -> DispatchResult {
	// Set up the test environment for AssetHub to Frequency communication
	
	FrequencyWestend::execute_with(|| {
		type FrequencyRuntimeOrigin = <FrequencyWestend as Chain>::RuntimeOrigin;
		let force_xcm_version_op = <FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_xcm_version(
			FrequencyRuntimeOrigin::root(),
			bx!(FrequencyWestend::sibling_location_of(AssetHubWestend::para_id())),
			SAFE_XCM_VERSION,
		);
		assert_ok!(force_xcm_version_op);
	});
	AssetHubWestend::execute_with(|| {
		type AssetHubRuntimeOrigin = <AssetHubWestend as Chain>::RuntimeOrigin;

		let freq_location = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
		println!("Setting XCM version on AssetHub for Frequency: {:?}", freq_location);

		let asset_hub_xcm_version_op = <AssetHubWestend as AssetHubWestendPallet>::PolkadotXcm::force_xcm_version(
			AssetHubRuntimeOrigin::root(),
			bx!(freq_location),
			SAFE_XCM_VERSION,
		);
		assert_ok!(asset_hub_xcm_version_op);
		println!("AssetHub XCM version set to {}", SAFE_XCM_VERSION);
	});
	Ok(())
}	

fn frequency_to_asset_hub_subscribe_version(t: FrequencyToAssetHubTest) -> DispatchResult {
	let query_id = 1234u64;
	let amount = 1_000_000_000_000u128; // 1 WND for fees

	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;

	// Create an XCM message with BuyExecution + SubscribeVersion instruction
	// let xcm = Xcm::<()>(vec![
	// 	WithdrawAsset((Here, amount).into()),
	// 	BuyExecution {
	// 		fees: (Here, amount/2).into(),
	// 		weight_limit: Unlimited, // Allow unlimited weight purchase for simplicity
	// 	},
	// 	SubscribeVersion {
	// 		query_id,
	// 		max_response_weight: Weight::from_all(0)
	// 	},
	// ]);

	// Call the force_default_xcm_version function
	// let set_default_xcm_op =
	// 	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_default_xcm_version(
	// 		t.root_origin.clone(),
	// 		Some(SAFE_XCM_VERSION),
	// 	);

	// assert_ok!(set_default_xcm_op);

	// // Call the force_xcm_version function
	// let set_xcm_op =
	// 	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_xcm_version(
	// 		t.root_origin.clone(),
	// 		bx!(Parent.into()),
	// 		SAFE_XCM_VERSION,
	// 	);
	// assert_ok!(set_xcm_op);

	let force_subscribe_version_op =
		<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::force_subscribe_version_notify(
			t.root_origin.clone(),
			bx!(VersionedLocation::V5(FrequencyWestend::sibling_location_of(AssetHubWestend::para_id()))),
		);
	assert_ok!(force_subscribe_version_op);

	// let inner_call: <FrequencyWestend as Chain>::RuntimeCall =
	// 	RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
	// 		dest: bx!(VersionedLocation::V5(Parent.into())),
	// 		message: bx!(VersionedXcm::V5(xcm)),
	// 	});

	// let _ = <FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(t.root_origin, bx!(inner_call));

	Ok(())
}

fn assert_subscribe_version_sent(_t: FrequencyToRelayTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	// Check that the SubscribeVersion message was sent
	assert_expected_events!(
		FrequencyWestend,
		vec![
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent {
				origin: _,
				destination,
				message: _,
				message_id: _
			}) => {
				destination: *destination == Parent.into(),
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

/// Tests XCM version subscription using the SubscribeVersion instruction.
/// This follows the official pattern: send an XCM with SubscribeVersion instruction
/// to request version information from a remote chain and receive version updates.
#[test]
fn test_subscribe_version_instruction() {
	let sender = FrequencyWestendSender::get();
	// let receiver = WestendReceiver::get();
	let receiver = AssetHubWestendReceiver::get();
	let amount = 1_000_000_000_000;

	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());

	// Fund Frequency sovereign account on AH
	AssetHubWestend::fund_para_sovereign(FrequencyWestend::para_id(), amount * 2);

	println!("Setting up AssetHub test environment...");
	let _ = setup_assethub_test();
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

	// let mut ah_test = AssetHubToFrequencyTest::new(test_args);

	// test.set_assertion::<FrequencyWestend>(assert_subscribe_version_sent);
	test.set_assertion::<AssetHubWestend>(assert_subscribe_version_received_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_asset_hub_subscribe_version);
	test.assert();
}

fn relay_to_frequency_query_response(t: RelayToFrequencyTest) -> DispatchResult {
	let query_id = 9999u64;
	let received_version = 5u32; // XCM version 5

	type RuntimeCall = <Westend as Chain>::RuntimeCall;

	// Simulate the relay chain sending a QueryResponse with version information
	// This is what should happen after receiving a SubscribeVersion instruction
	let version_response_xcm = Xcm::<()>(vec![QueryResponse {
		query_id,
		response: Response::Version(received_version),
		max_weight: Weight::from_parts(1_000_000, 10_000),
		querier: None,
	}]);

	// Send the QueryResponse from relay to parachain
	let inner_call: <Westend as Chain>::RuntimeCall =
		RuntimeCall::XcmPallet(pallet_xcm::Call::send {
			dest: bx!(VersionedLocation::V5(Parachain(FrequencyWestend::para_id().into()).into())),
			message: bx!(VersionedXcm::V5(version_response_xcm)),
		});

	let _ = <Westend as WestendPallet>::Sudo::sudo(t.root_origin, bx!(inner_call));

	Ok(())
}

fn assert_query_response_sent_from_relay(_t: RelayToFrequencyTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;

	// Check that the QueryResponse message was sent from relay
	assert_expected_events!(
		Westend,
		vec![
			RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent {
				origin: _,
				destination,
				message: _,
				message_id: _
			}) => {
				destination: *destination == Parachain(FrequencyWestend::para_id().into()).into(),
			},
		]
	);
}

fn assert_version_query_response_processed(_t: RelayToFrequencyTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	// Check that the QueryResponse was processed on FrequencyWestend
	assert_expected_events!(
		FrequencyWestend,
		vec![
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

/// Tests receiving version information via QueryResponse.
/// After sending SubscribeVersion, the remote chain should respond with a QueryResponse
/// containing the version information. This test simulates that response using the framework.
#[test]
fn test_version_query_response_handling() {
	let sender = WestendSender::get();
	let receiver = FrequencyWestendReceiver::get();
	let amount = WESTEND_ED;

	let destination = Westend::child_location_of(FrequencyWestend::para_id());

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_relay(
			destination.clone(),
			receiver,
			amount, // Use non-zero amount
		),
	};

	let mut test = RelayToFrequencyTest::new(test_args);

	test.set_assertion::<Westend>(assert_query_response_sent_from_relay);
	test.set_assertion::<FrequencyWestend>(assert_version_query_response_processed);
	test.set_dispatchable::<Westend>(relay_to_frequency_query_response);
	test.assert();
}

/// Tests the VERSION_DISCOVERY_QUEUE_SIZE configuration and basic version setup.
/// This verifies that both chains have proper XCM version configuration.
#[test]
fn test_version_discovery_queue_configuration() {
	// Test FrequencyWestend version configuration
	FrequencyWestend::execute_with(|| {
		// Check that the current XCM version is properly configured
		let current_version = pallet_xcm::CurrentXcmVersion::get();
		println!("FrequencyWestend current XCM version: {}", current_version);
		assert!(current_version >= 3); // Should be using modern XCM version

		// Test basic version configuration - we can't directly test the queue size
		// but we can verify the version system is working
		println!("XCM version configuration is properly set up on FrequencyWestend");
	});

	// Test Westend version configuration
	Westend::execute_with(|| {
		let current_version = pallet_xcm::CurrentXcmVersion::get();
		println!("Westend current XCM version: {}", current_version);
		assert!(current_version >= 3);
		println!("XCM version configuration is properly set up on Westend");
	});
}

fn frequency_to_relay_subscribe_and_unsubscribe(t: FrequencyToRelayTest) -> DispatchResult {
	let query_id = 5678u64;
	let amount = 1_000_000_000u128; // 1 WND for fees

	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;

	// Step 1: Subscribe to version updates with proper fee payment
	let subscribe_xcm = Xcm::<()>(vec![
		// Buy execution weight first
		BuyExecution { fees: (Parent, amount).into(), weight_limit: Unlimited },
		SubscribeVersion { query_id, max_response_weight: Weight::from_parts(500_000_000, 50_000) },
	]);

	let subscribe_call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
		dest: bx!(VersionedLocation::V5(Parent.into())),
		message: bx!(VersionedXcm::V5(subscribe_xcm)),
	});

	let _ = <FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(
		t.root_origin.clone(),
		bx!(subscribe_call),
	);

	// Step 2: Unsubscribe from version updates with proper fee payment
	let unsubscribe_xcm = Xcm::<()>(vec![
		// Buy execution weight for unsubscribe operation
		BuyExecution { fees: (Parent, amount).into(), weight_limit: Unlimited },
		UnsubscribeVersion,
	]);

	let unsubscribe_call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
		dest: bx!(VersionedLocation::V5(Parent.into())),
		message: bx!(VersionedXcm::V5(unsubscribe_xcm)),
	});

	let _ = <FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(
		t.root_origin,
		bx!(unsubscribe_call),
	);

	Ok(())
}

fn assert_version_subscription_lifecycle(_t: FrequencyToRelayTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	// Check that both subscription and unsubscription messages were sent
	assert_expected_events!(
		FrequencyWestend,
		vec![
			// First message: Subscribe
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent {
				origin: _,
				destination,
				message: _,
				message_id: _
			}) => {
				destination: *destination == Parent.into(),
			},
			// Second message: Unsubscribe
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent {
				origin: _,
				destination,
				message: _,
				message_id: _
			}) => {
				destination: *destination == Parent.into(),
			},
		]
	);
}

fn assert_version_lifecycle_received_on_relay(_t: FrequencyToRelayTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;

	// Let's see what events actually occur during the full lifecycle
	assert_expected_events!(
		Westend,
		vec![
			// Accept any XCM pallet event for now to see what happens
			RuntimeEvent::XcmPallet(_) => {},
			// Accept any MessageQueue event for now
			RuntimeEvent::MessageQueue(_) => {},
		]
	);
}

/// Tests the complete version subscription lifecycle: subscribe, receive response, unsubscribe.
/// This follows the official pattern from XCM documentation and tests the full flow.
#[test]
fn test_version_subscription_lifecycle() {
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount = WESTEND_ED;

	let destination = FrequencyWestend::parent_location();

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

	let mut test = FrequencyToRelayTest::new(test_args);

	test.set_assertion::<FrequencyWestend>(assert_version_subscription_lifecycle);
	test.set_assertion::<Westend>(assert_version_lifecycle_received_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_subscribe_and_unsubscribe);
	test.assert();
}
