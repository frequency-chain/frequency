use crate::foreign_balance_on;
use crate::imports::*;

pub const ASSET_MIN_BALANCE: u128 = 1;

fn frequency_location_as_seen_by_asset_hub() -> Location {
	AssetHubWestend::sibling_location_of(FrequencyWestend::para_id())
}

fn find_fee_asset_item(assets: Assets, fee_asset_id: AssetId) -> u32 {
	assets
		.into_inner()
		.iter()
		.position(|a| a.id == fee_asset_id)
		.expect("Fee asset not found in asset list") as u32
}

fn build_fee_and_value_assets(fee_dot: Balance, xrqcy_value: Balance) -> Vec<Asset> {
	vec![
		(Parent, fee_dot).into(), // DOT - used as fee
		(frequency_location_as_seen_by_asset_hub(), xrqcy_value).into(), // XRQCY used as main transfer asset
	]
}

fn build_asset_hub_to_frequency_test(
	sender: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	receiver: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	destination: Location,
	frqcy_amount: Balance,
	assets: Assets,
	fee_asset_item: u32,
) -> AssetHubToFrequencyTest {
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination,
			receiver,
			frqcy_amount,
			assets.clone(),
			None,
			fee_asset_item,
		),
	};

	AssetHubToFrequencyTest::new(test_args)
}

fn assert_sender_assets_burned_correctly(t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	let frequency_location = frequency_location_as_seen_by_asset_hub();
	let (_, xrqcy_teleport_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	let frequency_sibling_account =
		AssetHubWestend::sovereign_account_id_of(frequency_location.clone());

	let (_, total_fee) = fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();
	let remote_execution_fee: u128 = total_fee / 2;

	AssetHubWestend::assert_xcm_pallet_attempted_complete(None);
	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Frequency burned for teleportation
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Burned { asset_id, owner, balance }
			) => {
				asset_id: *asset_id == frequency_location,
				owner: *owner == t.sender.account_id,
				balance: *balance == xrqcy_teleport_amount,
			},
			// Remote fee burned
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, amount }) => {
				who: *who == t.sender.account_id,
				amount: *amount == total_fee,
			},
			// Sovereign account funded for remote fee
			RuntimeEvent::Balances(pallet_balances::Event::Minted { who, amount }) => {
				who: *who == frequency_sibling_account,
				amount: *amount == remote_execution_fee,
			},
		]
	);
}

fn assert_receiver_fee_burned_and_asset_minted(t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	FrequencyWestend::assert_xcmp_queue_success(None);

	let (_, total_fee) = fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();
	let remote_execution_fee: u128 = total_fee / 2;

	let (_, xrqcy_teleport_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Mint remaining fees and deposit them into receiver otherwise
			RuntimeEvent::Balances(
				pallet_balances::Event::Minted { who, amount }
			) => {
				who: *who == FrequencyWestendReceiver::get(),
				amount: *amount == xrqcy_teleport_amount,
			},
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Issued { asset_id, owner, amount },
			) => {
				asset_id: *asset_id == Parent.into(),
				owner: *owner == t.receiver.account_id,
				amount: *amount < remote_execution_fee,
			},
		]
	);
}

fn mint_xrqcy_on_asset_hub(
	beneficiary: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	amount_to_mint: Balance,
) {
	type ForeignAssets = <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets;
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;

	AssetHubWestend::execute_with(|| {
		let signed_origin =
			<AssetHubWestend as Chain>::RuntimeOrigin::signed(FrequencyAssetOwner::get());

		assert_ok!(ForeignAssets::mint(
			signed_origin,
			frequency_location_as_seen_by_asset_hub().into(),
			beneficiary.clone().into(),
			amount_to_mint
		));

		assert_expected_events!(
			AssetHubWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Issued { asset_id, owner, amount }
				) => {
					asset_id: *asset_id == frequency_location_as_seen_by_asset_hub().into(),
					owner: *owner == beneficiary.clone().into(),
					amount: *amount == amount_to_mint,
				},
			]
		);
	});
}

fn mint_dot_on_frequency(
	beneficiary: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	amount_to_mint: Balance,
) {
	type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	FrequencyWestend::execute_with(|| {
		let signed_origin =
			<FrequencyWestend as Chain>::RuntimeOrigin::signed(FrequencyAssetOwner::get());

		assert_ok!(ForeignAssets::mint(
			signed_origin,
			Parent.into(),
			beneficiary.clone().into(),
			amount_to_mint
		));

		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Issued { asset_id, owner, amount }
				) => {
					asset_id: *asset_id == Parent.into(),
					owner: *owner == beneficiary.clone().into(),
					amount: *amount == amount_to_mint,
				},
			]
		);
	});
}

fn create_dot_asset_on_frequency() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
		let sudo_origin = <FrequencyWestend as Chain>::RuntimeOrigin::root();

		ForeignAssets::force_create(
			sudo_origin,
			Parent.into(),
			FrequencyAssetOwner::get().into(),
			false,
			1u128.into(),
		);

		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::ForceCreated { asset_id, .. }
				) => {
					asset_id: *asset_id == Parent.into(),
				},
			]
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(Parent.into()));
	});
}

fn create_frequency_asset_on_ah() {
	let frequency_location_on_asset_hub = frequency_location_as_seen_by_asset_hub();

	AssetHubWestend::force_create_foreign_asset(
		frequency_location_on_asset_hub.clone().try_into().unwrap(),
		FrequencyAssetOwner::get(),
		false,
		ASSET_MIN_BALANCE,
		vec![],
	);
}

fn execute_xcm_asset_hub_to_frequency(t: AssetHubToFrequencyTest) -> DispatchResult {
	let all_assets = t.args.assets.clone().into_inner();
	let mut assets = all_assets.clone();

	let mut fees = assets.remove(t.args.fee_asset_item as usize);
	
	if let Fungible(fees_amount) = fees.fun {
		fees.fun = Fungible(fees_amount / 2);
	}
	// xcm to be executed at dest
	let xcm_on_dest = Xcm(vec![
		// since this is the last hop, we don't need to further use any assets previously
		// reserved for fees (there are no further hops to cover delivery fees for); we
		// RefundSurplus to get back any unspent fees
		RefundSurplus,
		DepositAsset { assets: Wild(All), beneficiary: t.args.beneficiary },
	]);
	let xcm = Xcm::<()>(vec![
		WithdrawAsset(all_assets.into()),
		PayFees { asset: fees.clone() },
		InitiateTransfer {
			destination: t.args.dest,
			remote_fees: Some(AssetTransferFilter::ReserveDeposit(fees.into())),
			preserve_origin: false,
			assets: BoundedVec::truncate_from(vec![AssetTransferFilter::Teleport(assets.into())]),
			remote_xcm: xcm_on_dest,
		},
	]);

	<AssetHubWestend as AssetHubWestendPallet>::PolkadotXcm::execute(
		t.signed_origin,
		bx!(staging_xcm::VersionedXcm::from(xcm.into())),
		Weight::MAX,
	)
	.unwrap();
	Ok(())
}

// ===========================================================================
// ======= DOT (fee) + xFRQCY (value) Transfer: AssetHub → Frequency =========
// ===========================================================================
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::asset_hub_to_frequency_transfer_xfrqcy_with_dot_fee -p frequency-westend-integration-tests -- --nocapture
#[test]
fn asset_hub_to_frequency_transfer_xfrqcy_with_dot_fee() {
	// ────────────────
	// Test Setup
	// ────────────────
	AssetHubWestend::fund_accounts(vec![(
		AssetHubWestendSender::get(),
		AssetHubExistentialDeposit::get() * 10_000,
	)]);
	let dot_fee_amount: Balance = AssetHubExistentialDeposit::get() * 3000;
	let xrqcy_amount = FrequencyExistentialDeposit::get() * 1000;
	let frequency_location_on_asset_hub = frequency_location_as_seen_by_asset_hub();

	create_dot_asset_on_frequency();

	create_frequency_asset_on_ah();
	mint_xrqcy_on_asset_hub(
		AssetHubWestendSender::get().clone(),
		(xrqcy_amount * 2).clone(),
	);

	let sender = AssetHubWestendSender::get();
	let receiver = FrequencyWestendReceiver::get();
	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	let assets: Assets =
		build_fee_and_value_assets(dot_fee_amount * 2, xrqcy_amount).into();
	let fee_asset_item = find_fee_asset_item(assets.clone(), AssetId(Parent.into()));

	// ────────────────────────────────
	//  Pre-dispatch State Snapshot
	// ────────────────────────────────
	let sender_balance_of_dot_on_frequency_before =
		foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);
	assert_eq!(sender_balance_of_dot_on_frequency_before, 0u128);

	let sender_balance_of_xrqcy_on_asset_hub_before =
		foreign_balance_on!(AssetHubWestend, frequency_location_on_asset_hub.clone(), &sender.clone());
	assert_eq!(sender_balance_of_xrqcy_on_asset_hub_before, 2_000_000_000u128);

	let sender_dot_before = AssetHubWestend::execute_with(|| {
		type Balances = <AssetHubWestend as AssetHubWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&sender)
	});
	assert_eq!(sender_dot_before, 14_096_000_000_000u128);

	let receiver_xrqcy_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&receiver)
	});

	assert_eq!(receiver_xrqcy_before, 4_096_000_000u128);

	// ─────────────────────────────
	// Build Test Context
	// ─────────────────────────────
	let mut test = build_asset_hub_to_frequency_test(
		sender.clone(),
		receiver.clone(),
		destination.clone(),
		xrqcy_amount,
		assets,
		fee_asset_item,
	);

	test.set_assertion::<AssetHubWestend>(assert_sender_assets_burned_correctly);
	test.set_assertion::<FrequencyWestend>(assert_receiver_fee_burned_and_asset_minted);
	test.set_dispatchable::<AssetHubWestend>(execute_xcm_asset_hub_to_frequency);
	test.assert();
}
