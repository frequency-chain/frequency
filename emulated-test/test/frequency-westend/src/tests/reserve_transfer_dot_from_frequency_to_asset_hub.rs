use crate::foreign_balance_on;
use crate::imports::*;

fn frequency_to_asset_hub_reserve_transfer_assets(t: FrequencyToAssetHubTest) -> DispatchResult {
	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

pub fn frequency_to_asset_hub_sender_assertions(t: FrequencyToAssetHubTest) {
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

fn frequency_to_asset_hub_receiver_assertions(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	let sov_frequency_on_ahr = AssetHubWestend::sovereign_account_id_of(
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id()),
	);
	AssetHubWestend::assert_xcmp_queue_success(None);
	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Amount to reserve transfer is burned from Parachain's Sovereign account
			RuntimeEvent::Assets(pallet_assets::Event::Burned { asset_id, owner, balance }) => {
				asset_id: *asset_id == RESERVABLE_ASSET_ID,
				owner: *owner == sov_frequency_on_ahr,
				balance: *balance == t.args.amount,
			},
			// Fee amount is burned from Parachain's Sovereign account
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, .. }) => {
				who: *who == sov_frequency_on_ahr,
			},
			// Amount to reserve transfer is issued for beneficiary
			RuntimeEvent::Assets(pallet_assets::Event::Issued { asset_id, owner, amount }) => {
				asset_id: *asset_id == RESERVABLE_ASSET_ID,
				owner: *owner == t.receiver.account_id,
				amount: *amount == t.args.amount,
			},
			// Remaining fee amount is minted for for beneficiary
			RuntimeEvent::Balances(pallet_balances::Event::Minted { who, .. }) => {
				who: *who == t.receiver.account_id,
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

const DOT_DOLLAR: u128 = 1_000_000_000_000;
const DOT_CENT: u128 = DOT_DOLLAR / 100;

// =========================================================================
// ======= Reserve Transfers - WSND Native Asset - Frequency<>AssetHub==========
// =========================================================================
/// Reserve Transfers of Frequency Native from Asset Hub to Frequency should work
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_from_frequency_to_asset_hub -p frequency-westend-integration-tests -- --nocapture
// transfer_type=DestinationReserve
#[test]
fn reserve_transfer_dot_from_frequency_to_asset_hub() {
	let amount_dot_to_send: Balance = AssetHubExistentialDeposit::get() * 1000;
	// assert_eq!(amount_to_send, 10_000 * DOLLAR);
	setup_foreign_asset_on_frequency_and_fund_ah_sov(amount_dot_to_send);

	/// 20K dollars
	let sender = FrequencyWestendSender::get();
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

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver.clone(),
			amount_dot_to_send,
			assets.clone(),
			None,
			0,
		),
	};

	let mut test = FrequencyToAssetHubTest::new(test_args);

	// Query initial balances

	let receiver_balance_before = test.receiver.balance;

	// Set assertions and dispatchables
	test.set_assertion::<FrequencyWestend>(frequency_to_asset_hub_sender_assertions);
	test.set_assertion::<AssetHubWestend>(frequency_to_asset_hub_receiver_assertions);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_asset_hub_reserve_transfer_assets);
	test.assert();

	// Query final balances
	let sender_dot_assets_after =
		foreign_balance_on!(FrequencyWestend, asset_hub_native_asset_location, &sender);

	let receiver_balance_after = test.receiver.balance;

	let frequency_sender_native_after = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&FrequencyWestendSender::get())
	});

	// Sender's balance is reduced by amount sent 
	assert!(sender_dot_assets_after < sender_dot_assets_before - amount_dot_to_send);
	// // Receiver's balance is increased
	assert!(receiver_balance_after > receiver_balance_before);
	// // Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// // `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// // should be non-zero
	assert!(receiver_balance_after < receiver_balance_before + amount_dot_to_send);
}
