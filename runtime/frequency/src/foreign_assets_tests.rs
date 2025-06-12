use super::*;
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungibles::Inspect, ChangeMembers},
};
use pallet_assets::Error as AssetsError;
use sp_runtime::{traits::BadOrigin, AccountId32};

// Test account constants
pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([2u8; 32]);
pub const DAVE: AccountId32 = AccountId32::new([3u8; 32]);
pub const EVE: AccountId32 = AccountId32::new([4u8; 32]);

mod mock {
	use super::*;
	use frame_support::{
		construct_runtime, derive_impl, parameter_types,
		traits::{ConstU128, ConstU32, ConstU64, Hooks},
		weights::Weight,
	};
	use pallet_balances::AccountData;
	use sp_runtime::{traits::IdentityLookup, BuildStorage};

	type Block = frame_system::mocking::MockBlock<Test>;
	type Balance = u128;
	type AccountId = AccountId32;
	type AssetId = u32;

	// Configure a mock runtime to test the pallet.
	construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Balances: pallet_balances,
			Assets: pallet_assets,
			Council: pallet_collective::<Instance1>,
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
		pub const ExistentialDeposit: Balance = 1;
		pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(Weight::MAX);
		pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(50) * BlockWeights::get().max_block;
	}

	#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
	impl frame_system::Config for Test {
		type Block = Block;
		type AccountId = AccountId;
		type AccountData = AccountData<Balance>;
		type Lookup = IdentityLookup<Self::AccountId>;
		type SS58Prefix = SS58Prefix;
	}

	#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
	impl pallet_balances::Config for Test {
		type AccountStore = System;
		type Balance = Balance;
		type ExistentialDeposit = ExistentialDeposit;
	}

	type CouncilCollective = pallet_collective::Instance1;

	// Custom CreateOrigin that allows root and council members
	pub struct ForeignAssetCreateOrigin;

	use frame_support::traits::{EnsureOrigin, EnsureOriginWithArg};

	// NOTE: Make sure this implementation matches the lib.rs implementation!!
	// u32 used in place of Location
	impl EnsureOriginWithArg<RuntimeOrigin, u32> for ForeignAssetCreateOrigin {
		type Success = AccountId;

		fn try_origin(o: RuntimeOrigin, _location: &u32) -> Result<Self::Success, RuntimeOrigin> {
			if let Ok(()) =
				<frame_system::EnsureRoot<AccountId> as EnsureOrigin<RuntimeOrigin>>::try_origin(
					o.clone(),
				) {
				// In test environment, return ALICE since Treasury pallet is not configured
				return Ok(ALICE);
			}

			if let Ok(who) = frame_system::ensure_signed(o.clone()) {
				let members =
					pallet_collective::Members::<Test, pallet_collective::Instance1>::get();
				if members.contains(&who) {
					return Ok(who);
				}
			}
			Err(o)
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin(_asset_id: &u32) -> Result<RuntimeOrigin, ()> {
			Ok(RuntimeOrigin::root())
		}
	}

	impl pallet_assets::Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type Balance = Balance;
		type AssetId = AssetId;
		type AssetIdParameter = AssetId;
		type Currency = Balances;
		type CreateOrigin = ForeignAssetCreateOrigin;
		type ForceOrigin = frame_system::EnsureRoot<AccountId>;
		type AssetDeposit = ConstU128<1>;
		type AssetAccountDeposit = ConstU128<10>;
		type MetadataDepositBase = ConstU128<1>;
		type MetadataDepositPerByte = ConstU128<1>;
		type ApprovalDeposit = ConstU128<1>;
		type StringLimit = ConstU32<50>;
		type Freezer = ();
		type Extra = ();
		type WeightInfo = ();
		type RemoveItemsLimit = ConstU32<1000>;
		type CallbackHandle = ();
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
		type Holder = ();
	}

	impl pallet_collective::Config<CouncilCollective> for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type Proposal = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type MotionDuration = ConstU64<5>;
		type MaxProposals = ConstU32<25>;
		type MaxMembers = ConstU32<10>;
		type DefaultVote = pallet_collective::PrimeDefaultVote;
		type WeightInfo = ();
		type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
		type MaxProposalWeight = MaxProposalWeight;
		type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
		type KillOrigin = frame_system::EnsureRoot<AccountId>;
		type Consideration = ();
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000), (DAVE, 1000), (EVE, 1000)],
			dev_accounts: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn advance_block() {
		System::on_finalize(System::block_number());
		Council::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Council::on_initialize(System::block_number());
	}

	pub fn set_council_members(members: Vec<AccountId>) {
		// Remove all current members (simulate full replacement)
		Council::change_members(&members, &[], members.clone());
		// Set the prime member if any
		Council::set_prime(members.get(0).cloned());
		// Advance the block to ensure collective pallet state is updated
		advance_block();
	}
}

use mock::{Council, RuntimeOrigin, System, *};

mod create_asset_tests {
	use super::*;

	#[test]
	fn create_asset_with_root_origin_should_succeed() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let min_balance = 1u128;

			assert_ok!(Assets::create(
				RuntimeOrigin::root(),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}

	#[test]
	fn create_asset_with_council_member_should_succeed() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let min_balance = 1u128;

			set_council_members(vec![BOB]);

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}

	#[test]
	fn create_asset_with_multiple_council_members_should_succeed() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let min_balance = 1u128;

			set_council_members(vec![BOB, CHARLIE]);

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(asset_id));

			let asset_id_2 = 101u32;
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(CHARLIE),
				asset_id_2.into(),
				ALICE,
				min_balance.into()
			));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(asset_id_2));
		});
	}

	#[test]
	fn create_asset_with_regular_account_should_fail() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let min_balance = 1u128;

			set_council_members(vec![BOB, CHARLIE]);

			assert_noop!(
				Assets::create(
					RuntimeOrigin::signed(DAVE),
					asset_id.into(),
					ALICE,
					min_balance.into()
				),
				BadOrigin
			);
			assert!(!<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}

	#[test]
	fn create_asset_after_council_membership_change() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let asset_id_2 = 101u32;
			let min_balance = 1u128;

			set_council_members(vec![BOB]);
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));

			set_council_members(vec![CHARLIE]);

			assert_noop!(
				Assets::create(
					RuntimeOrigin::signed(BOB),
					asset_id_2.into(),
					ALICE,
					min_balance.into()
				),
				BadOrigin
			);

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(CHARLIE),
				asset_id_2.into(),
				ALICE,
				min_balance.into()
			));
		});
	}

	#[test]
	fn create_asset_with_empty_council_should_fail() {
		new_test_ext().execute_with(|| {
			let asset_id = 100u32;
			let min_balance = 1u128;

			set_council_members(vec![]);

			assert_noop!(
				Assets::create(
					RuntimeOrigin::signed(BOB),
					asset_id.into(),
					ALICE,
					min_balance.into()
				),
				BadOrigin
			);
		});
	}
}

mod force_create_asset_tests {
	use super::*;

	#[test]
	fn force_create_asset_with_root_origin_should_succeed() {
		new_test_ext().execute_with(|| {
			let asset_id = 200u32;
			let is_sufficient = true;
			let min_balance = 1u128;

			// Root origin should be able to force create assets
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				asset_id.into(),
				ALICE,
				is_sufficient,
				min_balance.into()
			));

			// Verify asset was created
			assert!(<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}

	#[test]
	fn force_create_asset_with_council_member_should_fail() {
		new_test_ext().execute_with(|| {
			let asset_id = 200u32;
			let is_sufficient = true;
			let min_balance = 1u128;

			// Set up council
			set_council_members(vec![BOB]);

			// Council member should NOT be able to force create assets (only root can)
			assert_noop!(
				Assets::force_create(
					RuntimeOrigin::signed(BOB),
					asset_id.into(),
					ALICE,
					is_sufficient,
					min_balance.into()
				),
				BadOrigin
			);

			// Verify asset was not created
			assert!(!<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}

	#[test]
	fn force_create_asset_with_regular_account_should_fail() {
		new_test_ext().execute_with(|| {
			let asset_id = 200u32;
			let is_sufficient = true;
			let min_balance = 1u128;

			// Regular user should not be able to force create assets
			assert_noop!(
				Assets::force_create(
					RuntimeOrigin::signed(DAVE),
					asset_id.into(),
					ALICE,
					is_sufficient,
					min_balance.into()
				),
				BadOrigin
			);

			// Verify asset was not created
			assert!(!<Assets as Inspect<AccountId>>::asset_exists(asset_id));
		});
	}
}

mod edge_case_tests {
	use super::*;

	#[test]
	fn create_duplicate_asset_should_fail() {
		new_test_ext().execute_with(|| {
			let asset_id = 300u32;
			let min_balance = 1u128;

			set_council_members(vec![BOB]);

			// Create asset first time
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));

			// Try to create same asset again should fail
			assert_noop!(
				Assets::create(
					RuntimeOrigin::signed(BOB),
					asset_id.into(),
					ALICE,
					min_balance.into()
				),
				AssetsError::<Test>::InUse
			);
		});
	}

	#[test]
	fn create_asset_with_different_parameters() {
		new_test_ext().execute_with(|| {
			set_council_members(vec![ALICE, BOB, CHARLIE, DAVE, EVE]);

			// Test with different admin accounts
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				100u32.into(),
				ALICE,
				1u128.into()
			));

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				101u32.into(),
				BOB,
				10u128.into()
			));

			// Test with different min_balance values
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				102u32.into(),
				ALICE,
				100u128.into()
			));

			// All assets should exist
			assert!(<Assets as Inspect<AccountId>>::asset_exists(100u32));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(101u32));
			assert!(<Assets as Inspect<AccountId>>::asset_exists(102u32));
		});
	}

	#[test]
	fn council_size_one_requirement() {
		new_test_ext().execute_with(|| {
			let asset_id = 400u32;
			let min_balance = 1u128;

			// Test that at least 1 council member is required (EnsureMembers<_, _, 1>)
			set_council_members(vec![BOB]);

			// Single council member should be sufficient
			assert_ok!(Assets::create(
				RuntimeOrigin::signed(BOB),
				asset_id.into(),
				ALICE,
				min_balance.into()
			));

			// Add more members and test they all work
			set_council_members(vec![BOB, CHARLIE, DAVE]);

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(CHARLIE),
				401u32.into(),
				ALICE,
				min_balance.into()
			));

			assert_ok!(Assets::create(
				RuntimeOrigin::signed(DAVE),
				402u32.into(),
				ALICE,
				min_balance.into()
			));
		});
	}
}
