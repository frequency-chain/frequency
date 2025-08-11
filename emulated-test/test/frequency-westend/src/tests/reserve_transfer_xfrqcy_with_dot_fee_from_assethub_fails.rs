use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{
		create_frequency_asset_on_ah, ensure_dot_asset_exists_on_frequency, mint_xrqcy_on_asset_hub,
	},
};
use crate::tests::utils::{build_assethub_to_frequency_test, find_fee_asset_item};

fn frequency_location_as_seen_by_asset_hub() -> Location {
	AssetHubWestend::sibling_location_of(FrequencyWestend::para_id())
}

fn asset_hub_location_as_seen_by_frequency() -> Location {
	FrequencyWestend::sibling_location_of(AssetHubWestend::para_id())
}

fn assert_sender_assets_burned_correctly(t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	let frequency_location = frequency_location_as_seen_by_asset_hub();
	let (_, xrqcy_reserve_amount) =
		non_fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();

	let frequency_sibling_account =
		AssetHubWestend::sovereign_account_id_of(frequency_location.clone());

	let (_, total_fee) = fee_asset(&t.args.assets, t.args.fee_asset_item as usize).unwrap();
	let remote_execution_fee: u128 = total_fee / 2;

	AssetHubWestend::assert_xcm_pallet_attempted_complete(None);
	assert_expected_events!(
		AssetHubWestend,
		vec![
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Burned { asset_id, owner, balance }
			) => {
				asset_id: *asset_id == frequency_location,
				owner: *owner == t.sender.account_id,
				balance: *balance == xrqcy_reserve_amount,
			},
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Issued { asset_id, owner, amount }
			) => {
				asset_id: *asset_id == frequency_location,
				owner: *owner == frequency_sibling_account,
				amount: *amount == xrqcy_reserve_amount,
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

fn assert_receiver_errors(_t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	assert_expected_events!(
		FrequencyWestend,
		vec![
			// Mint remaining fees and deposit them into receiver otherwise
			RuntimeEvent::PolkadotXcm(
				pallet_xcm::Event::ProcessXcmError { error, origin,.. },
			) => {
				origin: *origin == asset_hub_location_as_seen_by_frequency(),
				error: *error == XcmError::UntrustedReserveLocation,
			},
			RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success, .. }) => {
				success: *success == false,
			},
		]
	);
}

fn execute_reserve_transfer_xcm_asset_hub_to_frequency(
	t: AssetHubToFrequencyTest,
) -> DispatchResult {
	let all_assets = t.args.assets.clone().into_inner();
	let mut assets = all_assets.clone();

	let mut fees = assets.remove(t.args.fee_asset_item as usize);

	if let Fungible(fees_amount) = fees.fun {
		fees.fun = Fungible(fees_amount / 2);
	}

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
			assets: BoundedVec::truncate_from(vec![AssetTransferFilter::ReserveDeposit(
				assets.into(),
			)]),
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
// ======= DOT (fee) + xFRQCY (value) Transfer: AssetHub → Frequency Success =========
// ===========================================================================
// Teleporting for AssetHub to Frequency fails because it checking account
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_xfrqcy_with_dot_fee_from_assethub_fails -- --nocapture
#[test]
fn reserve_transfer_xfrqcy_with_dot_fee_from_assethub_fails() {
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

	// Fund checking account
	FrequencyWestend::fund_accounts(vec![(FrequencyCheckingAccount::get(), xrqcy_teleport_amount)]);
	create_frequency_asset_on_ah();
	mint_xrqcy_on_asset_hub(
		AssetHubWestendSender::get().clone(),
		(xrqcy_teleport_amount * 2).clone(),
	);

	let sender = AssetHubWestendSender::get();
	let receiver = FrequencyWestendReceiver::get();
	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	let assets: Assets = vec![
		(Parent, dot_fee_amount).into(), // DOT - used as fee
		(frequency_location_as_seen_by_asset_hub(), xrqcy_teleport_amount).into(), // XRQCY used as main transfer asset
	].into();
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

	let sender_dot_on_assethub_before = assethub_balance_of(&sender);
	assert_eq!(sender_dot_on_assethub_before, 4_196_000_000_000u128);

	let receiver_xrqcy_on_frequency_before = frequency_balance_of(&receiver);
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
	test.set_assertion::<FrequencyWestend>(assert_receiver_errors);
	test.set_dispatchable::<AssetHubWestend>(execute_reserve_transfer_xcm_asset_hub_to_frequency);
	test.assert();
}
