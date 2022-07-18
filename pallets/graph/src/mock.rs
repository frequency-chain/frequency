use crate as pallet_graph;
use crate::Permission;
use codec::Encode;
use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		GraphPallet: pallet_graph::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MaxNodes: u32 = 100_000;
	pub const MaxFollows: u32 = 10000;
}

impl pallet_graph::Config for Test {
	type Event = Event;
	type MaxNodes = MaxNodes;
	type MaxFollows = MaxFollows;
	type WeightInfo = ();
	// type MaxFollowers = u64;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn test_add_node() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_eq!(GraphPallet::node_count(), 1);
		assert_eq!(GraphPallet::edge_count(), 0);
	});
}

#[test]
fn follow3_unfollow3() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 3));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1240));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 2767378));

		assert_ok!(GraphPallet::follow_child_public(Origin::signed(account_id_origin), 1234, 3));
		assert_ok!(GraphPallet::follow_child_public(Origin::signed(account_id_origin), 1234, 1240));
		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			2767378
		));

		assert_ok!(GraphPallet::follow_child_public(Origin::signed(account_id_origin), 1240, 3));

		let perm = GraphPallet::read_from_child_tree(1234, GraphPallet::get_storage_key(3));
		assert_eq!(perm, Some(Permission { data: 1 }.encode().to_vec()));

		GraphPallet::read_all_keys(1234);

		assert_ok!(GraphPallet::unfollow_child_public(Origin::signed(account_id_origin), 1234, 3));
		let perm = GraphPallet::read_from_child_tree(1234, GraphPallet::get_storage_key(3));
		assert_eq!(perm, None);
	});
}
