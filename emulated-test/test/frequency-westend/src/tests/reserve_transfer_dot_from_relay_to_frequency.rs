use crate::foreign_balance_on;
use crate::imports::*;
use westend_system_emulated_network::westend_emulated_chain::westend_runtime::Dmp;

fn relay_to_frequency_receiver_assertions(t: RelayToFrequencyTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	assert_expected_events!(
		FrequencyWestend,
		vec![
			RuntimeEvent::ForeignAssets(pallet_assets::Event::Issued { asset_id, owner, .. }) => {
				asset_id: *asset_id == WestendLocation::get(),
				owner: *owner == t.receiver.account_id,
			},
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

fn relay_to_frequency_reserve_transfer_assets(t: RelayToFrequencyTest) -> DispatchResult {
	let Junction::Parachain(para_id) = *t.args.dest.chain_location().last().unwrap() else {
		unimplemented!("Destination is not a parachain?")
	};

	Dmp::make_parachain_reachable(para_id);
	<Westend as WestendPallet>::XcmPallet::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

fn relay_to_frequency_sender_assertions(t: RelayToFrequencyTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;

	Westend::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(297_174_000, 6_196)));

	assert_expected_events!(
		Westend,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == Westend::sovereign_account_id_of(
					t.args.dest.clone()
				),
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn setup_foreign_asset_on_frequency() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		let sender = FrequencyWestendSender::get();

		let _ = <ForeignAssets as FungiblesCreate<_>>::create(
			Parent.into(),
			sender.clone(),
			false,
			1u32.into(),
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(Parent.into()));
	});
}


// =========================================================================
// ========= Reserve Transfers - DOT Asset - Relay<>Frequency ===========
// =========================================================================
/// Reserve Transfers of DOT from Relay to Parachain should work
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_from_relay_to_frequency -p frequency-westend-integration-tests -- --nocapture
// transfer_type=DestinationReserve
#[test]
fn reserve_transfer_dot_asset_from_relay_to_frequency() {
    setup_foreign_asset_on_frequency();
    let destination = Westend::child_location_of(FrequencyWestend::para_id());
    let sender = WestendSender::get();
    let amount_to_send: Balance = WESTEND_ED * 1000;

    let relay_native_asset_location = WestendLocation::get();
    let receiver = FrequencyWestendReceiver::get();

    let test_args = TestContext {
        sender,
        receiver: receiver.clone(),
        args: TestArgs::new_relay(destination.clone(), receiver.clone(), amount_to_send),

    };

    let mut test = RelayToFrequencyTest::new(test_args);

    let sender_balance_before = test.sender.balance;
    let receiver_assets_before = foreign_balance_on!(FrequencyWestend, relay_native_asset_location.clone(), &receiver);

    test.set_assertion::<Westend>(relay_to_frequency_sender_assertions);
    test.set_assertion::<FrequencyWestend>(relay_to_frequency_receiver_assertions);
    test.set_dispatchable::<Westend>(relay_to_frequency_reserve_transfer_assets);
    test.assert();

    let sender_balance_after = test.sender.balance;
	let receiver_assets_after =
		foreign_balance_on!(FrequencyWestend, relay_native_asset_location, &receiver);

	// Sender's balance is reduced by amount sent plus delivery fees
	assert!(sender_balance_after < sender_balance_before - amount_to_send);
	// Receiver's asset balance is increased
	assert!(receiver_assets_after > receiver_assets_before);
	// Receiver's asset balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	assert!(receiver_assets_after < receiver_assets_before + amount_to_send);

}
