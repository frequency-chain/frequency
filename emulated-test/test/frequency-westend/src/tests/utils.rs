use crate::imports::*;

pub fn ensure_dot_asset_exists_on_frequency() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
		let sudo_origin = <FrequencyWestend as Chain>::RuntimeOrigin::root();

		let _ = ForeignAssets::force_create(
			sudo_origin,
			Parent.into(),
			FrequencyAssetOwner::get().into(),
			true,
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

pub const ASSET_MIN_BALANCE: u128 = 1;

pub fn create_frequency_asset_on_ah() {
	let frequency_location_on_ah =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

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

pub fn mint_dot_on_frequency(
	beneficiary: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	amount_to_mint: Balance,
) {
	type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	let dot_location_as_seen_by_frequency: Location = Parent.into();

	FrequencyWestend::execute_with(|| {
		let signed_origin =
			<FrequencyWestend as Chain>::RuntimeOrigin::signed(FrequencyAssetOwner::get());

		assert_ok!(ForeignAssets::mint(
			signed_origin,
			dot_location_as_seen_by_frequency.clone().into(),
			beneficiary.clone().into(),
			amount_to_mint
		));

		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Issued { asset_id, owner, amount }
				) => {
					asset_id: *asset_id == dot_location_as_seen_by_frequency.clone().into(),
					owner: *owner == beneficiary.clone().into(),
					amount: *amount == amount_to_mint,
				},
			]
		);
	});
}

pub fn mint_xrqcy_on_asset_hub(
	beneficiary: AccountIdOf<<AssetHubWestend as Chain>::Runtime>,
	amount_to_mint: Balance,
) {
	type ForeignAssets = <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets;
	type RuntimeEvent = <AssetHubWestend as Chain>::RuntimeEvent;
	let frequency_location_as_seen_by_asset_hub =
		AssetHubWestend::sibling_location_of(FrequencyWestend::para_id());

	AssetHubWestend::execute_with(|| {
		let signed_origin =
			<AssetHubWestend as Chain>::RuntimeOrigin::signed(FrequencyAssetOwner::get());

		assert_ok!(ForeignAssets::mint(
			signed_origin,
			frequency_location_as_seen_by_asset_hub.clone().into(),
			beneficiary.clone().into(),
			amount_to_mint
		));

		assert_expected_events!(
			AssetHubWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Issued { asset_id, owner, amount }
				) => {
					asset_id: *asset_id == frequency_location_as_seen_by_asset_hub.clone().into(),
					owner: *owner == beneficiary.clone().into(),
					amount: *amount == amount_to_mint,
				},
			]
		);
	});
}
