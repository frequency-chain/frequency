use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{
		create_frequency_asset_on_ah, ensure_dot_asset_exists_on_frequency, mint_dot_on_frequency,
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

fn build_fee_and_value_assets(fee_dot: Balance, native_token: Balance) -> Vec<Asset> {
	vec![
		(Parent, fee_dot).into(),    // DOT - used as fee
		(Here, native_token).into(), // XRQCY used as main transfer asset
	]
}

fn build_frequency_to_asset_hub_test(
	sender: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	receiver: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	destination: Location,
	frqcy_amount: Balance,
	assets: Assets,
	fee_asset_item: u32,
) -> FrequencyToAssetHubTest {
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(destination, receiver, frqcy_amount, assets, None, fee_asset_item),
	};

	FrequencyToAssetHubTest::new(test_args)
}

pub fn fund_sov_frequency_on_assethub(amount: Balance) {
	let frequency_location_on_ah =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sov_account = AssetHubWestend::sovereign_account_id_of(frequency_location_on_ah);
	AssetHubWestend::fund_accounts(vec![(sov_account.into(), amount)]);
}

fn execute_xcm_frequency_to_asset_hub(t: FrequencyToAssetHubTest) -> DispatchResult {
	let assets: Assets = t.args.assets.clone();

	let local_teleportable_asset: Asset =
		non_fee_asset(&assets, t.args.fee_asset_item as usize).unwrap().into();
	// TODO(https://github.com/paritytech/polkadot-sdk/issues/6197): dry-run to get exact fees.
	// For now )just use half the fees locally, half on dest

	// Use half of the fees to cover remote execution and the
	// remainding to cover delivery fees
	let mut remote_execution_fee_asset: Asset =
		fee_asset(&assets, t.args.fee_asset_item as usize).unwrap().into();
	if let Fungible(fees_amount) = remote_execution_fee_asset.fun {
		remote_execution_fee_asset.fun = Fungible(fees_amount / 2);
	}

	let xcm_on_dest = Xcm(vec![
		RefundSurplus,
		DepositAsset { assets: Wild(All), beneficiary: t.args.beneficiary },
	]);

	let xcm = Xcm::<()>(vec![
		WithdrawAsset(assets),
		// PayFees { asset: remote_fees.clone() },
		InitiateTransfer {
			destination: t.args.dest,
			remote_fees: Some(AssetTransferFilter::ReserveWithdraw(
				remote_execution_fee_asset.into(),
			)),
			preserve_origin: false,
			assets: BoundedVec::truncate_from(vec![AssetTransferFilter::Teleport(
				local_teleportable_asset.into(),
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
	let (_, xrqcy_teleport_amount) =
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
				amount: *amount == xrqcy_teleport_amount,
			},
		]
	);
}

fn assert_receiver_fee_burned_and_asset_minted(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	AssetHubWestend::assert_xcmp_queue_success(None);
	let sovereign_account_of_frequency =
		AssetHubWestend::sovereign_account_id_of(frequency_location_as_seen_by_asset_hub());

	let frequency_location = frequency_location_as_seen_by_asset_hub();

	let (_, total_fee) = fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();
	let remote_execution_fee: u128 = total_fee / 2;

	let (_, xrqcy_teleport_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Withdraw the fee amount from sov account and burn it
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned {  who, amount }
			) => {
				 who: *who == sovereign_account_of_frequency,
				 amount: *amount == remote_execution_fee,
				},
			// Issue the xFRQCY amount
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Issued { asset_id, owner, amount }
			) => {
				asset_id: *asset_id == frequency_location,
				owner: *owner == AssetHubWestendReceiver::get(),
				amount: *amount == xrqcy_teleport_amount,
			},
			// Mint remaining fees and deposit them into receiver otherwise
			RuntimeEvent::Balances(
				pallet_balances::Event::Minted { who, .. }
			) => {
				who: *who == AssetHubWestendReceiver::get(),
			},
		]
	);
}

// ===========================================================================
// ======= DOT (fee) + xFRQCY (value) Transfer: Frequency → AssetHub =========
// ===========================================================================
/// This test transfers xFRQCY from the Frequency parachain to AssetHub,
/// using DOT as the fee asset for both delivery and remote execution.
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::teleport_xfrqcy_to_assethub_with_dot_fee -p frequency-westend-integration-tests -- --nocapture
#[test]
fn teleport_xfrqcy_to_assethub_with_dot_fee() {
	// ────────────────
	// Test Setup
	// ────────────────
	let dot_fee_amount: Balance = AssetHubExistentialDeposit::get() * 1000;
	let xrqcy_transfer_amount = FrequencyExistentialDeposit::get() * 1000;
	let sender = FrequencyWestendSender::get();

	ensure_dot_asset_exists_on_frequency();
	mint_dot_on_frequency(sender.clone(), dot_fee_amount * 2);
	fund_sov_frequency_on_assethub(dot_fee_amount * 2);
	create_frequency_asset_on_ah();

	let receiver = AssetHubWestendReceiver::get();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let frequency_location_on_ah = frequency_location_as_seen_by_asset_hub();

	// Local fee amount(in DOT) should cover
	// 1. delivery cost to AH
	// 2. execution cost on AH
	let assets: Assets = build_fee_and_value_assets(dot_fee_amount, xrqcy_transfer_amount).into();
	let fee_asset_item = find_fee_asset_item(assets.clone(), AssetId(Parent.into()));

	// ────────────────────────────────
	//  Pre-dispatch State Snapshot
	// ────────────────────────────────
	let sender_balance_of_dot_on_frequency_before =
		foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);
	assert_eq!(sender_balance_of_dot_on_frequency_before, 2_000_000_000_000u128);

	let frequency_sender_native_before = frequency_balance_of(&sender);
	assert_eq!(frequency_sender_native_before, 4_096_000_000u128);

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
		xrqcy_transfer_amount,
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

	let sender_balance_of_dot_on_frequency_after =
		foreign_balance_on!(FrequencyWestend, Parent.into(), &sender);

	let receiver_balance_of_xfrqcy_on_ah_after =
		foreign_balance_on!(AssetHubWestend, frequency_location_on_ah.clone(), &receiver);

	assert!(
		sender_balance_of_dot_on_frequency_after <
			sender_balance_of_dot_on_frequency_before + dot_fee_amount
	);

	assert_eq!(
		receiver_balance_of_xfrqcy_on_ah_after, xrqcy_transfer_amount,
		"Sender's balance on AH does not equal the transfer amount"
	);
}
