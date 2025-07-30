use crate::{foreign_balance_on, imports::*, tests::utils::ensure_dot_asset_exists_on_frequency};

fn dispatch_asset_hub_to_frequency(t: AssetHubToFrequencyTest) -> DispatchResult {
	<AssetHubWestend as AssetHubWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

pub fn assert_receiver_on_frequency(t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

	FrequencyWestend::assert_xcmp_queue_success(None);
	for asset in t.args.assets.into_inner().into_iter() {
		let expected_id = asset.id.0.try_into().unwrap();
		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(pallet_assets::Event::Issued { asset_id, owner, .. }) => {
					asset_id: *asset_id == expected_id,
					owner: *owner == t.receiver.account_id,
				},
			]
		);
	}
}

pub fn assert_sender_from_asset_hub(t: AssetHubToFrequencyTest) {
	use pallet_balances::Event as BalancesEvent;
	use pallet_xcm::Event as XcmEvent;

	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;

	AssetHubWestend::assert_xcm_pallet_attempted_complete(None);

	let sov_acc_of_dest = AssetHubWestend::sovereign_account_id_of(t.args.dest.clone());

	for asset in t.args.assets.into_inner() {
		let asset_amount = match asset.fun {
			Fungible(amount) => amount,
			_ => panic!("Non-fungible assets are not supported in this test."),
		};

		match asset.id.0.unpack() {
			// Native asset (e.g., DOT)
			(1, []) => {
				assert_expected_events!(
					AssetHubWestend,
					vec![
						RuntimeEvent::Balances(BalancesEvent::Transfer { from, to, amount }) => {
							from: *from == t.sender.account_id,
							to: *to == sov_acc_of_dest,
							amount: *amount == asset_amount,
						},
					]
				);
			},

			_ => {
				panic!("Only DOT transfers are supported in this test.")
			},
		}
	}

	// Assert delivery fees were paid
	assert_expected_events!(
		AssetHubWestend,
		vec![
			RuntimeEvent::PolkadotXcm(XcmEvent::FeesPaid { .. }) => {},
		]
	);

	AssetHubWestend::assert_xcm_pallet_sent();
}

// =========================================================================
// ======= Reserve Transfers - WSND Native Asset - AssetHub<>Frequency==========
// =========================================================================
/// Reserve Transfers of Frequency Native from Asset Hub to Frequency should work
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::reserve_transfer_dot_from_asset_hub -- --nocapture
#[test]
fn reserve_transfer_dot_from_asset_hub() {
	ensure_dot_asset_exists_on_frequency();

	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sender = AssetHubWestendSender::get();
	let amount_to_send: Balance = AssetHubExistentialDeposit::get() * 2000;
	let assets_to_send: Assets = (Parent, amount_to_send).into();

	// Init values for Parachain
	let dot_asset_location = WestendLocation::get();
	let receiver = FrequencyWestendReceiver::get();

	let test_args = TestContext {
		sender,
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver.clone(),
			amount_to_send,
			assets_to_send.clone(),
			None,
			0,
		),
	};
	let mut test = AssetHubToFrequencyTest::new(test_args);

	// Query initial balances
	let sender_balance_before = test.sender.balance;

	let receiver_assets_before =
		foreign_balance_on!(FrequencyWestend, dot_asset_location.clone(), &receiver);

	assert!(
		receiver_assets_before == 0u128,
		"Expected receiver to start with zero DOT on Frequency"
	);

	test.set_assertion::<AssetHubWestend>(assert_sender_from_asset_hub);
	test.set_assertion::<FrequencyWestend>(assert_receiver_on_frequency);
	test.set_dispatchable::<AssetHubWestend>(dispatch_asset_hub_to_frequency);
	test.assert();

	let sender_balance_after = test.sender.balance;

	let receiver_assets_after =
		foreign_balance_on!(FrequencyWestend, dot_asset_location, &receiver);

	assert!(
		sender_balance_after < sender_balance_before - amount_to_send,
		"Sender's balance was NOT reduced by amount sent plus delivery fees"
	);

	assert!(receiver_assets_after > receiver_assets_before, "Receiver's assets did NOT increased");

	// Receiver's assets increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	assert!(receiver_assets_after < receiver_assets_before + amount_to_send, "Receiver's assets increased more than expected (delivery or execution fees may be missing)");
}
