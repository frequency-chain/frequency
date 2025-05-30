use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{
		create_frequency_asset_on_ah, ensure_dot_asset_exists_on_frequency, mint_xrqcy_on_asset_hub,
	},
};

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

fn build_fee_and_value_assets(fee_dot: Balance, xrqcy_teleport_amount: Balance) -> Vec<Asset> {
	vec![
		(Parent, fee_dot).into(), // DOT - used as fee
		(frequency_location_as_seen_by_asset_hub(), xrqcy_teleport_amount).into(), // XRQCY used as main transfer asset
	]
}

fn build_assethub_to_frequency_test(
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
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	FrequencyWestend::assert_xcmp_queue_success(None);

	let (_, total_fee) = fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();
	let remote_execution_fee: u128 = total_fee / 2;

	let (_, xrqcy_teleport_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	assert_expected_events!(
		FrequencyWestend,
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

fn execute_xcm_asset_hub_to_frequency(t: AssetHubToFrequencyTest) -> DispatchResult {
	let all_assets = t.args.assets.clone().into_inner();
	let mut assets = all_assets.clone();

	let mut fees = assets.remove(t.args.fee_asset_item as usize);

	if let Fungible(fees_amount) = fees.fun {
		fees.fun = Fungible(fees_amount / 2);
	}
	// xcm to be executed at dest
	let xcm_on_dest = Xcm(vec![
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
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::teleport_xfrqcy_with_dot_fee_from_assethub -- --nocapture
#[test]
fn teleport_xfrqcy_with_dot_fee_from_assethub() {
	// ────────────────
	// Test Setup
	// ────────────────
	let dot_fee_amount: Balance = AssetHubExistentialDeposit::get() * 1000; // ONE_DOT =  1_000_000_000_000
	let xrqcy_teleport_amount = FrequencyExistentialDeposit::get() * 100; // ONE_XRQCY = 100_000_000
	let frequency_location_on_asset_hub = frequency_location_as_seen_by_asset_hub();

	AssetHubWestend::fund_accounts(vec![(
		AssetHubWestendSender::get(),
		AssetHubExistentialDeposit::get() * 100,
	)]);

	ensure_dot_asset_exists_on_frequency();
	create_frequency_asset_on_ah();
	mint_xrqcy_on_asset_hub(
		AssetHubWestendSender::get().clone(),
		(xrqcy_teleport_amount * 2).clone(),
	);

	let sender = AssetHubWestendSender::get();
	let receiver = FrequencyWestendReceiver::get();
	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	let assets: Assets = build_fee_and_value_assets(dot_fee_amount, xrqcy_teleport_amount).into();
	let fee_asset_item = find_fee_asset_item(assets.clone(), AssetId(Parent.into()));

	// ────────────────────────────────
	//  Pre-dispatch State Snapshot
	// ────────────────────────────────
	let sender_dot_on_frequency_before =
		foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);
	assert_eq!(sender_dot_on_frequency_before, 0u128);

	let sender_xrqcy_on_assethub_before = foreign_balance_on!(
		AssetHubWestend,
		frequency_location_on_asset_hub.clone(),
		&sender.clone()
	);

	assert_eq!(sender_xrqcy_on_assethub_before, 200_000_000u128);

	let sender_dot_on_assethub_before = AssetHubWestend::execute_with(|| {
		type Balances = <AssetHubWestend as AssetHubWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&sender)
	});
	assert_eq!(sender_dot_on_assethub_before, 4_196_000_000_000u128);

	let receiver_xrqcy_on_frequency_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&receiver)
	});

	assert_eq!(receiver_xrqcy_on_frequency_before, 4_096_000_000u128);

	// ─────────────────────────────
	// Build Test Context
	// ─────────────────────────────
	let mut test = build_assethub_to_frequency_test(
		sender.clone(),
		receiver.clone(),
		destination.clone(),
		xrqcy_teleport_amount,
		assets,
		fee_asset_item,
	);

	test.set_assertion::<AssetHubWestend>(assert_sender_assets_burned_correctly);
	test.set_assertion::<FrequencyWestend>(assert_receiver_fee_burned_and_asset_minted);
	test.set_dispatchable::<AssetHubWestend>(execute_xcm_asset_hub_to_frequency);
	test.assert();
}
