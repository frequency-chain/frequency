use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{ensure_dot_asset_exists_on_frequency, mint_dot_on_frequency},
};

fn dispatch_reserve_transfer_dot(t: FrequencyToAssetHubTest) -> DispatchResult {
	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

pub fn assert_sender_burns_dot_correctly(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	FrequencyWestend::assert_xcm_pallet_attempted_complete(None);
	for asset in t.args.assets.into_inner().into_iter() {
		let expected_id = asset.clone().id.0;
		let asset_amount = if let Fungible(a) = asset.fun { Some(a) } else { None }.unwrap();
		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Burned { asset_id, owner, balance }
				) => {
					asset_id: *asset_id == expected_id,
					owner: *owner == t.sender.account_id,
					balance: *balance == asset_amount,
				},
			]
		);
	}
}

fn assert_receiver_receives_dot_correctly(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	let sov_frequency_on_ahr = AssetHubWestend::sovereign_account_id_of(
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id()),
	);
	AssetHubWestend::assert_xcmp_queue_success(None);
	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Amount to reserve transfer is burned from Frequency's Sovereign account
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, amount, .. }) => {
				who: *who == sov_frequency_on_ahr,
				amount: *amount == t.args.amount,
			},
			// Remaining amount, minus fee, is minted for for beneficiary
			RuntimeEvent::Balances(pallet_balances::Event::Minted { who, amount }) => {
				who: *who == t.receiver.account_id,
				amount: *amount < t.args.amount,
			},
		]
	);
}

fn build_reserve_transfer_test(
	sender: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	receiver: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	destination: Location,
	amount: Balance,
	assets: Assets,
) -> FrequencyToAssetHubTest {
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(destination, receiver.clone(), amount, assets, None, 0),
	};
	FrequencyToAssetHubTest::new(test_args)
}

fn fund_sov_frequency_account_on_ah(amount: Balance) {
	// Fund Frequency sovereign account on AssetHub
	let frequency_location_as_seen_by_ahr =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sov_frequency_on_ahr =
		AssetHubWestend::sovereign_account_id_of(frequency_location_as_seen_by_ahr);

	AssetHubWestend::fund_accounts(vec![(sov_frequency_on_ahr.into(), amount * 2)]);
}

// =========================================================================
// ======= Reserve Transfers - WSND Native Asset - Frequency<>AssetHub==========
// =========================================================================
/// Reserve Transfers of Frequency Native from Asset Hub to Frequency should work
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_to_assethub -p frequency-westend-integration-tests -- --nocapture
#[test]
fn reserve_transfer_dot_to_assethub() {
	let sender = FrequencyWestendSender::get();
	let amount_dot_to_send: Balance = AssetHubExistentialDeposit::get() * 1000;

	ensure_dot_asset_exists_on_frequency();
	mint_dot_on_frequency(sender.clone(), amount_dot_to_send * 2);
	fund_sov_frequency_account_on_ah(amount_dot_to_send);

	let receiver = AssetHubWestendReceiver::get();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let assets: Assets = (Parent, amount_dot_to_send).into();
	let asset_hub_native_asset_location = WestendLocation::get();

	let sender_dot_assets_before = foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);

	assert_eq!(sender_dot_assets_before, 2_000_000_000_000u128);

	let frequency_sender_native_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&FrequencyWestendSender::get())
	});

	assert_eq!(frequency_sender_native_before, 4_096_000_000u128);

	let mut test = build_reserve_transfer_test(
		sender.clone(),
		receiver.clone(),
		destination.clone(),
		amount_dot_to_send,
		assets.clone(),
	);

	// Query initial balances

	let receiver_balance_before = test.receiver.balance;

	// Set assertions and dispatchables
	test.set_assertion::<FrequencyWestend>(assert_sender_burns_dot_correctly);
	test.set_assertion::<AssetHubWestend>(assert_receiver_receives_dot_correctly);
	test.set_dispatchable::<FrequencyWestend>(dispatch_reserve_transfer_dot);
	test.assert();

	// Query final balances
	let sender_dot_assets_after =
		foreign_balance_on!(FrequencyWestend, asset_hub_native_asset_location, &sender);

	let receiver_balance_after = test.receiver.balance;

	// Sender's balance is reduced by amount sent
	assert!(sender_dot_assets_after < sender_dot_assets_before - amount_dot_to_send);
	// // Receiver's balance is increased
	assert!(receiver_balance_after > receiver_balance_before);
	// // Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// // `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// // should be non-zero
	assert!(receiver_balance_after < receiver_balance_before + amount_dot_to_send);
}
