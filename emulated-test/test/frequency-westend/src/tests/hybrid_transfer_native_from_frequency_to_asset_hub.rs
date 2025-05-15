use crate::foreign_balance_on;
use crate::imports::*;

pub const ASSET_MIN_BALANCE: u128 = 1;

/// Teleport Transfers of native asset from Parachain to System Parachain should work
// ===========================================================================
// ======= Transfer - DOT + Frequency - Parachain->AssetHub ==========
// ===========================================================================
/// Transfers of dot plus frequency asset from some Parachain to AssetHub
/// while paying fees using dot.
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::setup_foreign_asset_on_frequency_and_fund_ah_sov -p frequency-westend-integration-tests -- --nocapture

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

fn create_frequency_asset_on_ah() {
	let frequency_location_as_seen_by_ahr =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	AssetHubWestend::force_create_foreign_asset(
		frequency_location_as_seen_by_ahr.clone().try_into().unwrap(),
		FrequencyAssetOwner::get(),
		false,
		ASSET_MIN_BALANCE,
		vec![],
	);

	AssetHubWestend::execute_with(|| {
		type ForeignAssets = <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets;

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(
			frequency_location_as_seen_by_ahr
		));
	});
}

// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::hybrid_transfer_native_from_frequency_to_asset_hub -p frequency-westend-integration-tests -- --nocapture
#[test]
fn hybrid_transfer_native_from_frequency_to_asset_hub() {
	let fee_dot_to_send: Balance = AssetHubExistentialDeposit::get() * 1000;
	setup_foreign_asset_on_frequency_and_fund_ah_sov(fee_dot_to_send);
	create_frequency_asset_on_ah();

	let frequency_location_as_seen_by_ahr =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	let sender = FrequencyWestendSender::get();
	let receiver = AssetHubWestendReceiver::get();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let native_asset_amount_to_send = FrequencyExistentialDeposit::get() * 1000;
	let assets: Vec<Asset> =
		vec![(Parent, fee_dot_to_send).into(), (Here, native_asset_amount_to_send).into()];

	let asset_hub_native_asset_location = WestendLocation::get();

	let sender_dot_assets_before = foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);

	assert_eq!(sender_dot_assets_before, 2_000_000_000_000u128);

	let fee_asset_id = AssetId(Parent.into());
	let fee_asset_item = assets.iter().position(|a| a.id == fee_asset_id).unwrap() as u32;

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver.clone(),
			native_asset_amount_to_send,
			assets.into(),
			None,
			fee_asset_item,
		),
	};

	let mut test = FrequencyToAssetHubTest::new(test_args);

	let frequency_sender_native_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&FrequencyWestendSender::get())
	});

	assert_eq!(frequency_sender_native_before, 4_096_000_000u128);

	let receiver_frequency_before =
		foreign_balance_on!(AssetHubWestend, frequency_location_as_seen_by_ahr, &sender);

	assert_eq!(receiver_frequency_before, 0u128);

	test.set_assertion::<FrequencyWestend>(para_to_system_para_sender_assertions);
}
