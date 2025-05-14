use crate::foreign_balance_on;
use crate::imports::*;
use westend_system_emulated_network::westend_emulated_chain::westend_runtime::Dmp;

fn frequency_to_relay_reserve_transfer_assets(t: FrequencyToRelayTest) -> DispatchResult {
	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

fn frequency_to_relay_receiver_assertions(t: FrequencyToRelayTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;
	let sov_penpal_on_relay =
		Westend::sovereign_account_id_of(Westend::child_location_of(FrequencyWestend::para_id()));

	Westend::assert_ump_queue_processed(
		true,
		Some(FrequencyWestend::para_id()),
		Some(Weight::from_parts(306305000, 7_186)),
	);

	assert_expected_events!(
		Westend,
		vec![
			// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned { who, amount }
			) => {
				who: *who == sov_penpal_on_relay.clone().into(),
				amount: *amount == t.args.amount,
			},
			RuntimeEvent::Balances(pallet_balances::Event::Minted { .. }) => {},
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

fn frequency_to_relay_sender_assertions(t: FrequencyToRelayTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	FrequencyWestend::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
		864_610_000,
		8_799,
	)));
	assert_expected_events!(
		FrequencyWestend,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Burned { asset_id, owner, balance, .. }
			) => {
				asset_id: *asset_id == WestendLocation::get(),
				owner: *owner == t.sender.account_id,
				balance: *balance == t.args.amount,
			},
		]
	);
}

fn setup_foreign_asset_on_frequency_and_fund_ah_sov(amount_to_send: Balance) {
	// Create and mint DOT-derived asset on Frequency
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		let sender = FrequencyWestendSender::get();

		let _ = <ForeignAssets as FungiblesCreate<_>>::create(
			Parent.into(),
			sender.clone(),
			false,
			1u32.into(),
		);

		let _ = <ForeignAssets as FungiblesMutate<_>>::mint_into(
			Parent.into(),
			&sender.clone(),
			amount_to_send * 2,
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(Parent.into()));
	});

	// Fund Frequency sovereign account on AssetHub
	let frequency_location_as_seen_by_ahr =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sov_frequency_on_ahr =
		AssetHubWestend::sovereign_account_id_of(frequency_location_as_seen_by_ahr);

	AssetHubWestend::fund_accounts(vec![(sov_frequency_on_ahr.into(), amount_to_send * 2)]);
}

// =========================================================================
// ========= Reserve Transfers - DOT Asset - Frequency<>Relay ===========
// =========================================================================
/// Reserve Transfers of DOT from Relay to Parachain should work
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_from_relay_to_frequency -p frequency-westend-integration-tests -- --nocapture
#[test]
fn reserve_transfer_dot_from_frequency_to_relay() {
	let destination = FrequencyWestend::parent_location();
	let sender = FrequencyWestendSender::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;
	let assets: Assets = (Parent, amount_to_send).into();
	// let asset_owner = FrequencyAssetOwner::get();
	let relay_native_asset_location = WestendLocation::get();

	setup_foreign_asset_on_frequency_and_fund_ah_sov(amount_to_send);

	let receiver = WestendReceiver::get();
	let frequency_location_as_seen_by_relay =
		Westend::child_location_of(FrequencyWestend::para_id());
	let sov_frequency_on_relay =
		Westend::sovereign_account_id_of(frequency_location_as_seen_by_relay);

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

	let sender_assets_before =
		foreign_balance_on!(FrequencyWestend, relay_native_asset_location.clone(), &sender);
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<FrequencyWestend>(frequency_to_relay_sender_assertions);
	test.set_assertion::<Westend>(frequency_to_relay_receiver_assertions);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_reserve_transfer_assets);
	test.assert();

	let sender_assets_after =
		foreign_balance_on!(FrequencyWestend, relay_native_asset_location, &sender);
	let receiver_balance_after = test.receiver.balance;

	// Sender's balance is reduced by amount sent plus delivery fees
	assert!(sender_assets_after < sender_assets_before - amount_to_send);

	assert!(receiver_balance_after > receiver_balance_before);

	// Receiver's asset balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
}
