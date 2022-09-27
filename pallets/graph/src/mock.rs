use crate as pallet_graph;
use crate::{GraphType, Permission, PrivatePage, PublicPage};
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
fn follow_unfollow_child_public() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;
		let friendship = Permission { data: 8 };
		let page = 0u16;

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 3));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1240));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 2767378));

		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			3,
			friendship,
			page
		));
		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			2767378,
			friendship,
			page
		));
		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			1240,
			friendship,
			page
		));

		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1240,
			3,
			friendship,
			page
		));

		let perm = GraphPallet::read_public_graph_node(
			1234,
			GraphPallet::get_storage_key(GraphType::Public, &friendship, page),
		);
		assert_eq!(perm, Some(PublicPage::try_from(vec![3, 1240, 2767378]).unwrap()));

		let keys = GraphPallet::read_public_graph(1234);
		for (k, v) in keys {
			println!("public {:?} -> {:?}", k, v);
		}

		assert_ok!(GraphPallet::unfollow_child_public(
			Origin::signed(account_id_origin),
			1234,
			3,
			friendship,
			page
		));
		let perm = GraphPallet::read_public_graph_node(
			1234,
			GraphPallet::get_storage_key(GraphType::Public, &friendship, page),
		);
		assert_eq!(perm, Some(PublicPage::try_from(vec![1240, 2767378]).unwrap()));
	});
}

#[test]
fn follow_unfollow_child_private() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;
		let friendship = Permission { data: 8 };
		let page = 0u16;
		let data: PrivatePage = PrivatePage::try_from(vec![1, 2, 3, 4]).unwrap();

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 3));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1240));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 2767378));

		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page,
			data.clone()
		));

		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page + 1,
			data
		));

		let perm = GraphPallet::read_private_graph_node(
			1234,
			GraphPallet::get_storage_key(GraphType::Private, &friendship, page),
		);
		assert_eq!(perm, Some(PrivatePage::try_from(vec![1, 2, 3, 4]).unwrap()));

		let keys = GraphPallet::read_private_graph(1234);
		for (k, v) in keys {
			println!("private {:?} -> {:?}", k, v);
		}

		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page + 1,
			PrivatePage::try_from(vec![]).unwrap()
		));

		let perm = GraphPallet::read_private_graph_node(
			1234,
			GraphPallet::get_storage_key(GraphType::Private, &friendship, page + 1),
		);
		assert_eq!(perm, None);
	});
}

#[test]
fn change_page_public_tests() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;
		let friendship = Permission { data: 8 };
		let page = 0u16;

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 3));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 2767378));

		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			3,
			friendship,
			page
		));
		assert_ok!(GraphPallet::follow_child_public(
			Origin::signed(account_id_origin),
			1234,
			2767378,
			friendship,
			page + 1
		));

		// this should remove page
		assert_ok!(GraphPallet::unfollow_child_public(
			Origin::signed(account_id_origin),
			1234,
			3,
			friendship,
			page
		));

		assert_ok!(GraphPallet::change_page_number(
			Origin::signed(account_id_origin),
			1234,
			GraphType::Public,
			friendship,
			1,
			0
		));

		let keys = GraphPallet::read_public_graph(1234);
		for (k, v) in keys {
			println!("after {:?} -> {:?}", k, v);
		}
	});
}

#[test]
fn change_page_private_tests() {
	new_test_ext().execute_with(|| {
		let account_id_origin: u64 = 10;
		let friendship = Permission { data: 8 };
		let page = 0u16;
		let data: PrivatePage = PrivatePage::try_from(vec![1, 2, 3, 4]).unwrap();

		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 1234));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 3));
		assert_ok!(GraphPallet::add_node(Origin::signed(account_id_origin), 2767378));

		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page,
			data.clone()
		));
		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page + 1,
			data.clone()
		));

		// this should remove page
		assert_ok!(GraphPallet::private_graph_update(
			Origin::signed(account_id_origin),
			1234,
			friendship,
			page,
			PrivatePage::default()
		));

		assert_ok!(GraphPallet::change_page_number(
			Origin::signed(account_id_origin),
			1234,
			GraphType::Private,
			friendship,
			1,
			0
		));

		let keys = GraphPallet::read_private_graph(1234);
		for (k, v) in keys {
			println!("after {:?} -> {:?}", k, v);
		}
	});
}
