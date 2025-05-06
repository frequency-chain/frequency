use crate::imports::*;

#[macro_export]
macro_rules! foreign_balance_on {
	( $chain:ident, $id:expr, $who:expr ) => {
		emulated_integration_tests_common::impls::paste::paste! {
			<$chain>::execute_with(|| {
				type ForeignAssets = <$chain as [<$chain Pallet>]>::ForeignAssets;
				<ForeignAssets as Inspect<_>>::balance($id, $who)
			})
		}
	};
}

fn system_para_to_para_reserve_transfer_assets(t: AssetHubToFrequencyTest) -> DispatchResult {
	<AssetHubWestend as AssetHubWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

pub fn system_para_to_para_receiver_assertions(t: AssetHubToFrequencyTest) {
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

pub fn system_para_to_para_sender_assertions(t: AssetHubToFrequencyTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	AssetHubWestend::assert_xcm_pallet_attempted_complete(None);

	let sov_acc_of_dest = AssetHubWestend::sovereign_account_id_of(t.args.dest.clone());
	for asset in t.args.assets.into_inner().into_iter() {
		let expected_id = asset.id.0.clone().try_into().unwrap();
		let asset_amount = if let Fungible(a) = asset.fun { Some(a) } else { None }.unwrap();
		if asset.id == AssetId(Location::new(1, [])) {
			assert_expected_events!(
				AssetHubWestend,
				vec![
					// Amount of native asset is transferred to Parachain's Sovereign account
					RuntimeEvent::Balances(
						pallet_balances::Event::Transfer { from, to, amount }
					) => {
						from: *from == t.sender.account_id,
						to: *to == sov_acc_of_dest,
						amount: *amount == asset_amount,
					},
				]
			);
		} else if matches!(
			asset.id.0.unpack(),
			(0, [PalletInstance(ASSETS_PALLET_ID), GeneralIndex(_)])
		) {
			assert_expected_events!(
				AssetHubWestend,
				vec![
					// Amount of trust-backed asset is transferred to Parachain's Sovereign account
					RuntimeEvent::Assets(
						pallet_assets::Event::Transferred { from, to, amount, .. },
					) => {
						from: *from == t.sender.account_id,
						to: *to == sov_acc_of_dest,
						amount: *amount == asset_amount,
					},
				]
			);
		} else {
			assert_expected_events!(
				AssetHubWestend,
				vec![
					// Amount of foreign asset is transferred to Parachain's Sovereign account
					RuntimeEvent::ForeignAssets(
						pallet_assets::Event::Transferred { asset_id, from, to, amount },
					) => {
						asset_id: *asset_id == expected_id,
						from: *from == t.sender.account_id,
						to: *to == sov_acc_of_dest,
						amount: *amount == asset_amount,
					},
				]
			);
		}
	}
	assert_expected_events!(
		AssetHubWestend,
		vec![
			// Delivery fees are paid
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::FeesPaid { .. }) => {},
		]
	);
	AssetHubWestend::assert_xcm_pallet_sent();
}

// =========================================================================
// ======= Reserve Transfers - Native Asset - AssetHub<>Parachain ==========
// =========================================================================
/// Reserve Transfers of native asset from Asset Hub to Parachain should work
#[test]
fn reserve_transfer_native_asset_from_asset_hub_to_para() {
	// Init values for Asset Hub
	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sender = AssetHubWestendSender::get();
	println!("destination: {:?}", destination);
	println!("sender: {:?}", sender);
	let amount_to_send: Balance = AssetHubExistentialDeposit::get() * 2000;
	let assets: Assets = (Parent, amount_to_send).into();
	println!("asserts {:?}", assets);

	// // Init values for Parachain
	let system_para_native_asset_location = WestendLocation::get();
	let receiver = FrequencyWestendReceiver::get();
	println!("receiver------------- {:?}", receiver);

	// // Init Test
	let test_args = TestContext {
		sender,
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver.clone(),
			amount_to_send,
			assets.clone(),
			None,
			0,
		),
	};
	let mut test = AssetHubToFrequencyTest::new(test_args);

	// Create dot on frequency
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		<ForeignAssets as FungiblesCreate<_>>::create(
			Parent.into(),
			FrequencyWestendSender::get(),
			false,
			1u32.into(),
		);
		// assert!(<ForeignAssets as Inspect<_>>::asset_exists(
		// 	ParentThen([Parachain(1000), Parachain(3), PalletInstance(5)].into()).into()));
		assert!(<ForeignAssets as Inspect<_>>::asset_exists(Parent.into()));
	});

	// Query initial balances
	let sender_balance_before = test.sender.balance;
	let receiver_assets_before =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location.clone(), &receiver);

	println!("sender_balance_before {:?}", sender_balance_before);
	// println!("receiver_assets_before {:?}", receiver_assets_before);
	// test.set_assertion::<AssetHubWestend>(system_para_to_para_sender_assertions);
	test.set_assertion::<FrequencyWestend>(system_para_to_para_receiver_assertions);
	test.set_dispatchable::<AssetHubWestend>(system_para_to_para_reserve_transfer_assets);
	test.assert();

	// // Query final balances
	let sender_balance_after = test.sender.balance;
	println!("sender_balance_after {:?}", sender_balance_after);
	let receiver_assets_after =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location.clone(), &receiver);

	// let taco =	FrequencyWestend::execute_with(|| {
	// 		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
	// 		<ForeignAssets as Inspect<_>>::balance(system_para_native_asset_location.clone(), &receiver)
	// 	});

	// println!("taco: {:?}------", taco);

	// Sender's balance is reduced by amount sent plus delivery fees
	assert!(sender_balance_after < sender_balance_before - amount_to_send);
	// Receiver's assets is increased
	// assert!(receiver_assets_after > receiver_assets_before);
	// Receiver's assets increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	// assert!(receiver_assets_after < receiver_assets_before + amount_to_send);
}

// Reserve Transfers of native asset from Parachain to Asset Hub should work
// #[test]
// fn reserve_transfer_native_asset_from_para_to_asset_hub() {
// 	// Init values for Parachain
// 	let destination = PenpalA::sibling_location_of(AssetHubWestend::para_id());
// 	let sender = PenpalASender::get();
// 	let amount_to_send: Balance = ASSET_HUB_WESTEND_ED * 1000;
// 	let assets: Assets = (Parent, amount_to_send).into();
// 	let system_para_native_asset_location = RelayLocation::get();
// 	let asset_owner = PenpalAssetOwner::get();

// 	// fund Parachain's sender account
// 	PenpalA::mint_foreign_asset(
// 		<PenpalA as Chain>::RuntimeOrigin::signed(asset_owner),
// 		system_para_native_asset_location.clone(),
// 		sender.clone(),
// 		amount_to_send * 2,
// 	);

// 	// Init values for Asset Hub
// 	let receiver = AssetHubWestendReceiver::get();
// 	let penpal_location_as_seen_by_ahr = AssetHubWestend::sibling_location_of(PenpalA::para_id());
// 	let sov_penpal_on_ahr =
// 		AssetHubWestend::sovereign_account_id_of(penpal_location_as_seen_by_ahr);

// 	// fund Parachain's SA on Asset Hub with the native tokens held in reserve
// 	AssetHubWestend::fund_accounts(vec![(sov_penpal_on_ahr.into(), amount_to_send * 2)]);

// 	// Init Test
// 	let test_args = TestContext {
// 		sender: sender.clone(),
// 		receiver: receiver.clone(),
// 		args: TestArgs::new_para(
// 			destination.clone(),
// 			receiver.clone(),
// 			amount_to_send,
// 			assets.clone(),
// 			None,
// 			0,
// 		),
// 	};
// 	let mut test = ParaToSystemParaTest::new(test_args);

// 	// Query initial balances
// 	let sender_assets_before =
// 		foreign_balance_on!(PenpalA, system_para_native_asset_location.clone(), &sender);
// 	let receiver_balance_before = test.receiver.balance;

// 	// Set assertions and dispatchables
// 	test.set_assertion::<PenpalA>(para_to_system_para_sender_assertions);
// 	test.set_assertion::<AssetHubWestend>(para_to_system_para_receiver_assertions);
// 	test.set_dispatchable::<PenpalA>(para_to_system_para_reserve_transfer_assets);
// 	test.assert();

// 	// Query final balances
// 	let sender_assets_after =
// 		foreign_balance_on!(PenpalA, system_para_native_asset_location, &sender);
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced by amount sent plus delivery fees
// 	assert!(sender_assets_after < sender_assets_before - amount_to_send);
// 	// Receiver's balance is increased
// 	assert!(receiver_balance_after > receiver_balance_before);
// 	// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
// 	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
// 	// should be non-zero
// 	assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
// }
