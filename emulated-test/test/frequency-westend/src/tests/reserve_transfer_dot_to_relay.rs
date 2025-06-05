use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{ensure_dot_asset_exists_on_frequency, mint_dot_on_frequency},
};

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

fn assert_receiver_minted_on_relay(t: FrequencyToRelayTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;
	let sov_frequency_on_relay =
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
				who: *who == sov_frequency_on_relay.clone().into(),
				amount: *amount == t.args.amount,
			},
			RuntimeEvent::Balances(pallet_balances::Event::Minted { .. }) => {},
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

fn assert_sender_burned_asset_on_frequency(t: FrequencyToRelayTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	// FrequencyWestend::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
	// 	864_610_000,
	// 	8_799,
	// )));
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

pub fn setup_parent_asset_on_frequency(
	account: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	amount: Balance,
) {
	ensure_dot_asset_exists_on_frequency();
	mint_dot_on_frequency(account, amount)
}

pub fn fund_sov_frequency_on_westend(amount: Balance) {
	let freq_location = Westend::child_location_of(FrequencyWestend::para_id());
	let sov_account = Westend::sovereign_account_id_of(freq_location);
	Westend::fund_accounts(vec![(sov_account.into(), amount)]);
}

// =========================================================================
// ========= Reserve Transfers - DOT Asset - Frequency<>Relay ===========
// =========================================================================
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_from_frequency_to_relay -p frequency-westend-integration-tests -- --nocapture
/// Tests reserve transfer of DOT from Frequency to Relay (Westend).
/// Ensures sovereign account on relay is debited and receiver is credited.
#[test]
fn reserve_transfer_dot_from_frequency_to_relay() {
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;

	setup_parent_asset_on_frequency(sender.clone(), amount_to_send * 2);
	fund_sov_frequency_on_westend(amount_to_send * 2);

	let assets: Assets = (Parent, amount_to_send).into();
	let destination = FrequencyWestend::parent_location();

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

	let sender_assets_before = foreign_balance_on!(FrequencyWestend, dot_location.clone(), &sender);

	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<FrequencyWestend>(assert_sender_burned_asset_on_frequency);
	test.set_assertion::<Westend>(assert_receiver_minted_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_reserve_transfer_assets);
	test.assert();

	let sender_assets_after = foreign_balance_on!(FrequencyWestend, dot_location, &sender);

	let receiver_balance_after = test.receiver.balance;

	// Sender's balance is reduced by amount sent plus delivery fees
	assert!(sender_assets_after < sender_assets_before - amount_to_send);

	assert!(receiver_balance_after > receiver_balance_before);

	// Receiver's asset balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
}
