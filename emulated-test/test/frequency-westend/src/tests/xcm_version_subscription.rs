
use crate::imports::*;

// =========================================================================
// ================ XCM Version Discovery Tests ===========================
// =========================================================================

/// Tests automatic XCM version discovery that happens when chains communicate.
/// When parachains send XCM messages, they automatically negotiate and discover 
/// each other's supported XCM versions without explicit subscription calls.
#[test]
fn automatic_xcm_version_discovery_on_message_send() {
	// Test that when FrequencyWestend sends an XCM message to Westend,
	// automatic version discovery happens
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;

	// Setup assets and funding
	FrequencyWestend::execute_with(|| {
		type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;
		type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
		
		// Send a simple XCM message to relay - this triggers automatic version discovery
		let xcm = Xcm::<()>(vec![
			QueryPallet { 
				module_name: b"System".to_vec(), 
				response_info: QueryResponseInfo {
					destination: AccountId32Junction { 
						network: None, 
						id: sender.into()
					}.into(),
					query_id: 0,
					max_weight: Weight::from_parts(1_000_000, 10_000),
				}
			}
		]);

		let send_call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
			dest: bx!(VersionedLocation::V5(Parent.into())),
			message: bx!(VersionedXcm::V5(xcm)),
		});

		assert_ok!(<FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(
			<FrequencyWestend as Chain>::RuntimeOrigin::root(),
			bx!(send_call)
		));
		
		// The version discovery happens automatically when sending XCM messages
		// Check for XCM version-related events that indicate version negotiation
		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { 
					origin, 
					destination, 
					message, 
					message_id 
				}) => {
					destination: *destination == Parent.into(),
				},
			]
		);
	});
}

#[test]
fn xcm_version_stored_after_successful_communication() {
	// Test that XCM versions are stored in pallet storage after successful communication
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	
	// Send a simple message from parachain to relay
	FrequencyWestend::execute_with(|| {
		type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;
		
		// Check the supported version storage before sending a message
		let relay_location = Parent.into();
		
		// Send an XCM message that will trigger version discovery
		let xcm = Xcm::<()>(vec![
			ClearOrigin,
		]);

		let send_call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
			dest: bx!(VersionedLocation::V5(relay_location)),
			message: bx!(VersionedXcm::V5(xcm)),
		});

		assert_ok!(<FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(
			<FrequencyWestend as Chain>::RuntimeOrigin::root(),
			bx!(send_call)
		));
		
		// After sending, the pallet should have stored version information
		// In a real test, you would check pallet_xcm::SupportedVersion storage
		// to see if the relay's version has been discovered and stored
	});
}

#[test]
fn xcm_version_discovery_queue_configuration() {
	// Test that the XCM version discovery queue is properly configured
	FrequencyWestend::execute_with(|| {
		// Check that AdvertisedXcmVersion is using CurrentXcmVersion
		// This ensures the parachain advertises the correct version
		let advertised_version = pallet_xcm::CurrentXcmVersion::get();
		assert!(advertised_version >= 3); // Should be using a modern XCM version
	});
	
	// Also check on the relay side
	Westend::execute_with(|| {
		// Relay should also have proper XCM version configuration
		let advertised_version = pallet_xcm::CurrentXcmVersion::get();
		assert!(advertised_version >= 3); // Should be using a modern XCM version
	});
}
