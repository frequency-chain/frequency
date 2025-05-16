use crate::foreign_balance_on;
use crate::imports::*;

pub const ASSET_MIN_BALANCE: u128 = 1;

/// Teleport Transfers of native asset from Parachain to System Parachain should work
// ===========================================================================
// ======= Transfer - DOT + Frequency - Parachain->AssetHub ==========
// ===========================================================================
/// Transfers of dot plus frequency asset from some Parachain to AssetHub
/// while paying fees using dot.
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::setup_foreign_dot_on_frequency_and_fund_sov_on_ah -p frequency-westend-integration-tests -- --nocapture

fn frequency_location_as_seen_by_asset_hub() -> Location {
	AssetHubWestend::sibling_location_of(FrequencyWestend::para_id())
}

fn find_fee_asset_item(assets: &[Asset], fee_asset_id: AssetId) -> u32 {
	assets
		.iter()
		.position(|a| a.id == fee_asset_id)
		.expect("Fee asset not found in asset list") as u32
}

fn setup_frequency_to_asset_hub_test_env(dot_to_send: Balance) {
	setup_foreign_dot_on_frequency_and_fund_sov_on_ah(dot_to_send);
	create_frequency_asset_on_ah();
}

fn build_fee_and_value_assets(fee_dot: Balance, native_token: Balance) -> Vec<Asset> {
	vec![
		(Parent, fee_dot).into(),    // DOT (foreign) - used as fee
		(Here, native_token).into(), // XRQCY (native) - used as main transfer asset
	]
}

fn build_frequency_to_asset_hub_test(
	sender: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	receiver: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	destination: Location,
	frqcy_amount: Balance,
	assets: Vec<Asset>,
	fee_asset_item: u32,
) -> FrequencyToAssetHubTest {
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination,
			receiver,
			frqcy_amount,
			assets.into(),
			None,
			fee_asset_item,
		),
	};

	FrequencyToAssetHubTest::new(test_args)
}

fn setup_foreign_dot_on_frequency_and_fund_sov_on_ah(amount_to_send: Balance) {
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
	let frequency_location_on_ah = frequency_location_as_seen_by_asset_hub();
	let sov_frequency_on_ahr = AssetHubWestend::sovereign_account_id_of(frequency_location_on_ah);

	AssetHubWestend::fund_accounts(vec![(sov_frequency_on_ahr.into(), amount_to_send * 2)]);
}

fn create_frequency_asset_on_ah() {
	let frequency_location_on_ah = frequency_location_as_seen_by_asset_hub();

	AssetHubWestend::force_create_foreign_asset(
		frequency_location_on_ah.clone().try_into().unwrap(),
		FrequencyAssetOwner::get(),
		false,
		ASSET_MIN_BALANCE,
		vec![],
	);

	AssetHubWestend::execute_with(|| {
		type ForeignAssets = <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets;

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(frequency_location_on_ah));
	});
}

fn execute_xcm_frequency_to_asset_hub(t: FrequencyToAssetHubTest) -> DispatchResult {
	let all_assets: Vec<Asset> = t.args.assets.clone().into_inner();
	let mut assets = all_assets.clone();

	let local_asset = assets.remove(t.args.fee_asset_item as usize);
	// TODO(https://github.com/paritytech/polkadot-sdk/issues/6197): dry-run to get exact fees.
	// For now )just use half the fees locally, half on dest
	let mut remote_fees = assets.remove(0usize);
	if let Fungible(fees_amount) = remote_fees.fun {
		remote_fees.fun = Fungible(fees_amount / 2);
	}

	let xcm_on_dest = Xcm(vec![
		RefundSurplus,
		DepositAsset { assets: Wild(All), beneficiary: t.args.beneficiary },
	]);
	let xcm = Xcm::<()>(vec![
		WithdrawAsset(all_assets.into()),
		// PayFees { asset: remote_fees.clone() },
		InitiateTransfer {
			destination: t.args.dest,
			remote_fees: Some(AssetTransferFilter::ReserveWithdraw(remote_fees.into())),
			preserve_origin: false,
			assets: BoundedVec::truncate_from(vec![AssetTransferFilter::Teleport(
				local_asset.into(),
			)]),
			remote_xcm: xcm_on_dest,
		},
		RefundSurplus,
		DepositAsset {
			assets: Wild(All),
			beneficiary: AccountId32Junction {
				network: None,
				id: FrequencyWestendSender::get().into(),
			}
			.into(),
		},
	]);

	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::execute(
		t.signed_origin,
		bx!(staging_xcm::VersionedXcm::from(xcm.into())),
		Weight::MAX,
	)
	.unwrap();
	Ok(())
}

fn assert_sender_assets_burned_correctly(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	let system_para_native_asset_location = WestendLocation::get();
	let (_, expected_asset_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	FrequencyWestend::assert_xcm_pallet_attempted_complete(None);
	assert_expected_events!(
		FrequencyWestend,
		vec![
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Burned { asset_id, owner, .. }
			) => {
				asset_id: *asset_id == system_para_native_asset_location,
				owner: *owner == t.sender.account_id,
			},
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, amount }) => {
				who: *who == t.sender.account_id,
			},
		]
	);
}

fn assert_receiver_fee_burned_and_asset_minted(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	AssetHubWestend::assert_xcmp_queue_success(None);
	let sovereign_account_of_frequency =
		AssetHubWestend::sovereign_account_id_of(frequency_location_as_seen_by_asset_hub());

	let (_, remote_fee) = non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	assert_expected_events!(
		AssetHubWestend,
		vec![
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned {  who, amount }
			) => {
				 who: *who == sovereign_account_of_frequency,
				 amount: *amount == remote_fee / 2,
				},
		]
	);

	// # burn the fee
	// # mint frequency token
	// # create account account on
}

// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::hybrid_transfer_native_from_frequency_to_asset_hub -p frequency-westend-integration-tests -- --nocapture
#[test]
fn hybrid_transfer_native_from_frequency_to_asset_hub() {
	// Local fee amount(in DOT) should cover
	// 1. execution cost on AH
	// 2. delivery cost to BH
	// 3. execution cost on BH
	// let local_fee_amount = 200_000_000_000;
	// Remote fee amount(in WETH) should cover execution cost on Ethereum
	// let remote_fee_amount = 4_000_000_000;

	// ────────────────
	// 🔧 Test Setup
	// ────────────────
	let dot_fee_amount: Balance = AssetHubExistentialDeposit::get() * 1000000;
	let frqcy_transfer_amount = FrequencyExistentialDeposit::get() * 1000;

	setup_frequency_to_asset_hub_test_env(dot_fee_amount);

	let sender = FrequencyWestendSender::get();
	let receiver = AssetHubWestendReceiver::get();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let frequency_location_on_ah = frequency_location_as_seen_by_asset_hub();

	let assets: Vec<Asset> = build_fee_and_value_assets(dot_fee_amount, frqcy_transfer_amount);
	let fee_asset_item = find_fee_asset_item(&assets, AssetId(Parent.into()));

	// ────────────────────────────────
	//  Pre-dispatch State Snapshot
	// ────────────────────────────────
	let sender_dot_assets_before = foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);

	let frequency_sender_native_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&sender)
	});

	assert_eq!(frequency_sender_native_before, 4_096_000_000u128);
	assert_eq!(sender_dot_assets_before, 2_000_000_000_000_000u128);

	let receiver_frequency_before =
		foreign_balance_on!(AssetHubWestend, frequency_location_on_ah.clone(), &sender);

	assert_eq!(receiver_frequency_before, 0u128);

	// ─────────────────────────────
	// Build Test Context
	// ─────────────────────────────
	let mut test = build_frequency_to_asset_hub_test(
		sender.clone(),
		receiver.clone(),
		destination.clone(),
		frqcy_transfer_amount,
		assets,
		fee_asset_item,
	);

	// ─────────────────────────────
	// Execute + Assert
	// ─────────────────────────────
	test.set_assertion::<FrequencyWestend>(assert_sender_assets_burned_correctly);
	test.set_assertion::<AssetHubWestend>(assert_receiver_fee_burned_and_asset_minted);
	test.set_dispatchable::<FrequencyWestend>(execute_xcm_frequency_to_asset_hub);
	test.assert();

	let sender_balance_after = test.sender.balance;

	let receiver_assets_after =
		foreign_balance_on!(AssetHubWestend, frequency_location_on_ah.clone(), &receiver);

	assert_eq!(
		receiver_assets_after, frqcy_transfer_amount,
		"Sender's balance was NOT reduced by amount sent plus delivery fees"
	);
}
