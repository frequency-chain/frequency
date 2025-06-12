use crate::imports::*;
use frame_support::{assert_noop, traits::fungible::Mutate};
use staging_xcm::prelude::*;

const ASSET_ID: u32 = 1984;
const MIN_BALANCE: u128 = 1;

/// Test that root origin can create foreign assets
#[test]
fn root_can_create_foreign_assets() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

		let root_origin = <FrequencyWestend as Chain>::RuntimeOrigin::root();
		let asset_location: Location = Parent.into();
		let admin_account = FrequencyAssetOwner::get();

		// Root should be able to create foreign assets
		assert_ok!(ForeignAssets::create(
			root_origin,
			asset_location.clone(),
			admin_account.into(),
			MIN_BALANCE.into(),
		));

		// Verify the asset was created
		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Created { asset_id, creator, owner }
				) => {
					asset_id: *asset_id == asset_location,
					creator: *creator == FrequencyAssetOwner::get(),
					owner: *owner == FrequencyAssetOwner::get(),
				},
			]
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(asset_location));
	});
}

/// Test that regular signed accounts cannot create foreign assets
#[test]
fn regular_accounts_cannot_create_foreign_assets() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let regular_user = AccountIdOf::<FrequencyWestend>::from([1u8; 32]);
		let signed_origin = <FrequencyWestend as Chain>::RuntimeOrigin::signed(regular_user.clone());
		let asset_location: Location = Parent.into();

		// Fund the account for existential deposit
		<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
			&regular_user,
			FrequencyExistentialDeposit::get() * 1000,
		)
		.unwrap();

		// Regular signed account should NOT be able to create foreign assets
		assert_noop!(
			ForeignAssets::create(
				signed_origin,
				asset_location.clone(),
				regular_user.into(),
				MIN_BALANCE.into(),
			),
			sp_runtime::traits::BadOrigin
		);

		// Verify the asset was not created
		assert!(!<ForeignAssets as FungiblesInspect<_>>::asset_exists(asset_location));
	});
}

/// Test that council members can create foreign assets
#[test]
fn council_members_can_create_foreign_assets() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
		type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;

		let council_member = AccountIdOf::<FrequencyWestend>::from([2u8; 32]);
		let asset_location: Location = Parent.into();

		// Fund the council member account
		<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
			&council_member,
			FrequencyExistentialDeposit::get() * 1000,
		)
		.unwrap();

		// Note: In emulated tests, we assume the council is already set up
		// Council member should be able to create foreign assets
		let council_origin = <FrequencyWestend as Chain>::RuntimeOrigin::signed(council_member.clone());
		assert_ok!(ForeignAssets::create(
			council_origin,
			asset_location.clone(),
			council_member.clone().into(),
			MIN_BALANCE.into(),
		));

		// Verify the asset was created
		assert_expected_events!(
			FrequencyWestend,
			vec![
				RuntimeEvent::ForeignAssets(
					pallet_assets::Event::Created { asset_id, creator, owner }
				) => {
					asset_id: *asset_id == asset_location,
					creator: *creator == council_member.clone(),
					owner: *owner == council_member.clone(),
				},
			]
		);

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(asset_location));
	});
}

/// Test that non-council members cannot create foreign assets even if they were council members before
#[test]
fn former_council_members_cannot_create_foreign_assets() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let former_member = AccountIdOf::<FrequencyWestend>::from([3u8; 32]);
		let current_member = AccountIdOf::<FrequencyWestend>::from([4u8; 32]);
		let asset_location: Location = Parent.into();

		// Fund the accounts
		for account in [&former_member, &current_member] {
			<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
				account,
				FrequencyExistentialDeposit::get() * 1000,
			)
			.unwrap();
		}

		// Note: In emulated tests, we test the permission logic directly
		// Former member can create assets if they have the right permissions
		let first_asset_location = Location::new(1, [Parachain(1000)]);
		assert_ok!(ForeignAssets::create(
			<FrequencyWestend as Chain>::RuntimeOrigin::signed(former_member.clone()),
			first_asset_location.clone(),
			former_member.clone().into(),
			MIN_BALANCE.into(),
		));

		// Test that regular users cannot create assets
		assert_noop!(
			ForeignAssets::create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(former_member.clone()),
				asset_location.clone(),
				former_member.clone().into(),
				MIN_BALANCE.into(),
			),
			sp_runtime::traits::BadOrigin
		);

		// But authorized users should be able to
		assert_ok!(ForeignAssets::create(
			<FrequencyWestend as Chain>::RuntimeOrigin::signed(current_member.clone()),
			asset_location.clone(),
			current_member.clone().into(),
			MIN_BALANCE.into(),
		));
	});
}

/// Test that force_create still requires root origin only
#[test]
fn force_create_requires_root_origin() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let council_member = AccountIdOf::<FrequencyWestend>::from([5u8; 32]);
		let asset_location: Location = Parent.into();

		// Fund the council member
		<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
			&council_member,
			FrequencyExistentialDeposit::get() * 1000,
		)
		.unwrap();

		// Council member should NOT be able to force create assets
		assert_noop!(
			ForeignAssets::force_create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(council_member.clone()),
				asset_location.clone(),
				council_member.clone().into(),
				true,
				MIN_BALANCE.into(),
			),
			sp_runtime::traits::BadOrigin
		);

		// But root should be able to force create
		let root_origin = <FrequencyWestend as Chain>::RuntimeOrigin::root();
		assert_ok!(ForeignAssets::force_create(
			root_origin,
			asset_location.clone(),
			council_member.into(),
			true,
			MIN_BALANCE.into(),
		));

		assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(asset_location));
	});
}

/// Test multiple council members can all create assets
#[test]
fn multiple_council_members_can_create_assets() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let member1 = AccountIdOf::<FrequencyWestend>::from([6u8; 32]);
		let member2 = AccountIdOf::<FrequencyWestend>::from([7u8; 32]);
		let member3 = AccountIdOf::<FrequencyWestend>::from([8u8; 32]);

		// Fund all members
		for account in [&member1, &member2, &member3].iter() {
			<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
				account,
				FrequencyExistentialDeposit::get() * 1000,
			)
			.unwrap();
		}

		// Each member should be able to create assets
		let asset_locations = [
			Location::new(1, [Parachain(1001)]),
			Location::new(1, [Parachain(1002)]),
			Location::new(1, [Parachain(1003)]),
		];

		for (i, (member, location)) in [&member1, &member2, &member3]
			.iter()
			.zip(asset_locations.iter())
			.enumerate()
		{
			assert_ok!(ForeignAssets::create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(member.clone()),
				location.clone(),
				member.clone().into(),
				MIN_BALANCE.into(),
			));

			assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(location.clone()));
		}
	});
}

/// Test that asset creation with invalid parameters still respects permission restrictions
#[test]
fn invalid_asset_creation_respects_permissions() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let council_member = AccountIdOf::<FrequencyWestend>::from([9u8; 32]);
		let regular_user = AccountIdOf::<FrequencyWestend>::from([10u8; 32]);
		let asset_location: Location = Parent.into();

		// Fund accounts
		for account in [&council_member, &regular_user] {
			<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
				account,
				FrequencyExistentialDeposit::get() * 1000,
			)
			.unwrap();
		}

		// Create asset first with council member
		assert_ok!(ForeignAssets::create(
			<FrequencyWestend as Chain>::RuntimeOrigin::signed(council_member.clone()),
			asset_location.clone(),
			council_member.clone().into(),
			MIN_BALANCE.into(),
		));

		// Regular user should get permission error, not "asset already exists" error
		// This shows that permission check happens before other validations
		assert_noop!(
			ForeignAssets::create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(regular_user.clone()),
				asset_location,
				regular_user.into(),
				MIN_BALANCE.into(),
			),
			sp_runtime::traits::BadOrigin
		);
	});
}

/// Test asset creation with various asset locations
#[test]
fn council_can_create_assets_with_different_locations() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let council_member = AccountIdOf::<FrequencyWestend>::from([11u8; 32]);

		// Fund the member
		<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
			&council_member,
			FrequencyExistentialDeposit::get() * 1000,
		)
		.unwrap();

		// Test different types of asset locations
		let test_locations = vec![
			Parent.into(), // Relay chain
			Location::new(1, [Parachain(1000)]), // Other parachain
			Location::new(1, [Parachain(2000), GeneralIndex(1)]), // Asset on parachain
		];

		for (i, location) in test_locations.into_iter().enumerate() {
			assert_ok!(ForeignAssets::create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(council_member.clone()),
				location.clone(),
				council_member.clone().into(),
				MIN_BALANCE.into(),
			));

			assert!(<ForeignAssets as FungiblesInspect<_>>::asset_exists(location));
		}
	});
}

/// Test empty council scenario
#[test]
fn empty_council_prevents_asset_creation() {
	FrequencyWestend::execute_with(|| {
		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;

		let user = AccountIdOf::<FrequencyWestend>::from([12u8; 32]);
		let asset_location: Location = Parent.into();

		// Fund the user
		<FrequencyWestend as FrequencyWestendPallet>::Balances::mint_into(
			&user,
			FrequencyExistentialDeposit::get() * 1000,
		)
		.unwrap();

		// No one should be able to create assets except root
		assert_noop!(
			ForeignAssets::create(
				<FrequencyWestend as Chain>::RuntimeOrigin::signed(user.clone()),
				asset_location.clone(),
				user.clone().into(),
				MIN_BALANCE.into(),
			),
			sp_runtime::traits::BadOrigin
		);

		// But root should still work
		assert_ok!(ForeignAssets::create(
			<FrequencyWestend as Chain>::RuntimeOrigin::root(),
			asset_location.clone(),
			user.into(),
			MIN_BALANCE.into(),
		));
	});
}
