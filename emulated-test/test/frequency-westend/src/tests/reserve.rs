use crate::foreign_balance_on;
use crate::imports::*;

fn para_to_system_para_reserve_transfer_assets(t: FrequencyToAssetHubTest) -> DispatchResult {
	<FrequencyWestend as FrequencyWestendPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

pub fn para_to_system_para_sender_assertions(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	FrequencyWestend::assert_xcm_pallet_attempted_complete(None);
	for asset in t.args.assets.into_inner().into_iter() {
		let expected_id = asset.id.0;
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

pub fn para_to_system_para_receiver_assertions(t: FrequencyToAssetHubTest) {
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	AssetHubWestend::assert_xcmp_queue_success(None);

	let sov_acc_of_frequency = AssetHubWestend::sovereign_account_id_of(t.args.dest.clone());
	for (idx, asset) in t.args.assets.into_inner().into_iter().enumerate() {
		let expected_id = asset.id.0.clone().try_into().unwrap();
		let asset_amount = if let Fungible(a) = asset.fun { Some(a) } else { None }.unwrap();
		if idx == t.args.fee_asset_item as usize {
			assert_expected_events!(
				AssetHubWestend,
				vec![
					// Amount of native is withdrawn from Parachain's Sovereign account
					RuntimeEvent::Balances(
						pallet_balances::Event::Burned { who, amount }
					) => {
						who: *who == sov_acc_of_frequency.clone().into(),
						amount: *amount == asset_amount,
					},
					RuntimeEvent::Balances(pallet_balances::Event::Minted { who, .. }) => {
						who: *who == t.receiver.account_id,
					},
				]
			);
		} else {
			assert_expected_events!(
				AssetHubWestend,
				vec![
					// Amount of foreign asset is transferred from Parachain's Sovereign account
					// to Receiver's account
					RuntimeEvent::ForeignAssets(
						pallet_assets::Event::Burned { asset_id, owner, balance },
					) => {
						asset_id: *asset_id == expected_id,
						owner: *owner == sov_acc_of_frequency,
						balance: *balance == asset_amount,
					},
					RuntimeEvent::ForeignAssets(
						pallet_assets::Event::Issued { asset_id, owner, amount },
					) => {
						asset_id: *asset_id == expected_id,
						owner: *owner == t.receiver.account_id,
						amount: *amount == asset_amount,
					},
				]
			);
		}
	}
	assert_expected_events!(
		AssetHubWestend,
		vec![
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
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
// ======= Reserve Transfers - WSND Native Asset - AssetHub<>Frequency==========
// =========================================================================
/// Reserve Transfers of native asset from Asset Hub to Frequency should work
#[test]
fn reserve_transfer_native_asset_from_asset_hub_to_para() {
	// RUST_LOG="xcm=trace,system::events=trace" cargo test -p frequency-westend-integration-tests -- --nocapture
	// sp_tracing::try_init_simple();
	let destination = AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sender = AssetHubWestendSender::get();
	let amount_to_send: Balance = AssetHubExistentialDeposit::get() * 2000;
	let assets: Assets = (Parent, amount_to_send).into();

	// Init values for Parachain
	let system_para_native_asset_location = WestendLocation::get();
	let receiver = FrequencyWestendReceiver::get();
	println!("receiver {:?}", receiver);

	let receiver_XRQCY_assets_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&receiver)
	});
	assert_eq!(receiver_XRQCY_assets_before, 4096000000u128);

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
		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(Parent.into()));
	});

	// Query initial balances
	let sender_balance_before = test.sender.balance;

	let receiver_assets_before =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location.clone(), &receiver);
	assert!(receiver_assets_before == 0u128);

	// println!("receiver_assets_before {:?}", receiver_assets_before);
	test.set_assertion::<AssetHubWestend>(system_para_to_para_sender_assertions);
	test.set_assertion::<FrequencyWestend>(system_para_to_para_receiver_assertions);
	test.set_dispatchable::<AssetHubWestend>(system_para_to_para_reserve_transfer_assets);
	test.assert();

	let receiver_XRQCY_assets_after = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&receiver)
	});
	assert_eq!(receiver_XRQCY_assets_after, 4096000000u128);

	let sender_balance_after = test.sender.balance;

	let receiver_assets_after =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location.clone(), &receiver);

	assert!(
		sender_balance_after < sender_balance_before - amount_to_send,
		"Sender's balance was NOT reduced by amount sent plus delivery fees"
	);

	assert!(receiver_assets_after > receiver_assets_before, "Receiver's assets did NOT increased");

	// Receiver's assets increased by `amount_to_send - delivery_fees - bought_execution`;
	// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// should be non-zero
	assert!(receiver_assets_after < receiver_assets_before + amount_to_send);
}

#[test]
fn reserve_transfer_native_asset_from_para_to_asset_hub() {
	sp_tracing::try_init_simple();
	let destination = FrequencyWestend::sibling_location_of(AssetHubWestend::para_id());
	let sender = FrequencyWestendSender::get();
	let amount_to_send: Balance = AssetHubExistentialDeposit::get() * 1000;
	let assets: Assets = (Parent, amount_to_send).into();
	let system_para_native_asset_location = WestendLocation::get();

	let frequency_sender_native_before = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&FrequencyWestendSender::get())
	});

	assert_eq!(frequency_sender_native_before, 4096000000u128);

	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		<ForeignAssets as FungiblesCreate<_>>::create(
			Parent.into(),
			FrequencyWestendSender::get(),
			false,
			1u32.into(),
		);

		<ForeignAssets as FungiblesMutate<_>>::mint_into(
			Parent.into(),
			&FrequencyWestendSender::get(),
			amount_to_send * 2,
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(Parent.into()));
	});

	// Init values for Asset Hub
	let receiver = AssetHubWestendReceiver::get();
	let frequency_location_as_seen_by_ahr =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());
	let sov_frequency_on_ahr =
		AssetHubWestend::sovereign_account_id_of(frequency_location_as_seen_by_ahr);

	// fund Parachain's SA on Asset Hub with the native tokens held in reserve
	AssetHubWestend::fund_accounts(vec![(sov_frequency_on_ahr.into(), amount_to_send * 2)]);

	// Init Test
	let test_args = TestContext {
		sender: sender.clone(),
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
	let mut test = FrequencyToAssetHubTest::new(test_args);

	// Query initial balances
	let sender_assets_before =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location.clone(), &sender);
	let receiver_balance_before = test.receiver.balance;

	assert_eq!(sender_assets_before, 2000000000000u128);

	// Set assertions and dispatchables
	test.set_assertion::<FrequencyWestend>(para_to_system_para_sender_assertions);
	// test.set_assertion::<AssetHubWestend>(para_to_system_para_receiver_assertions);
	test.set_dispatchable::<FrequencyWestend>(para_to_system_para_reserve_transfer_assets);
	test.assert();

	// Query final balances
	let sender_assets_after =
		foreign_balance_on!(FrequencyWestend, system_para_native_asset_location, &sender);
	let receiver_balance_after = test.receiver.balance;

	let frequency_sender_native_after = FrequencyWestend::execute_with(|| {
		type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
		<Balances as Inspect<_>>::balance(&FrequencyWestendSender::get())
	});


	// // Sender's balance is reduced by amount sent plus delivery fees
	assert!(sender_assets_after < sender_assets_before - amount_to_send);
	// // Receiver's balance is increased
	// assert!(receiver_balance_after > receiver_balance_before);
	// // Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
	// // `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown but
	// // should be non-zero
	// assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
}
