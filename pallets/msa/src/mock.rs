use crate as pallet_msa;
use common_primitives::{msa::MessageSourceId, utils::wrap_binary_data};
use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use sp_core::{sr25519, Encode, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentifyAccount, IdentityLookup, Verify},
	AccountId32, MultiSignature,
};

pub use pallet_msa::Call as MsaCall;

pub type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

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
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>},
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
	type AccountId = AccountId;
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
	pub const MaxKeys: u32 = 10;
}

impl pallet_msa::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type ConvertIntoAccountId32 = ConvertInto;
	type MaxKeys = MaxKeys;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

pub fn test_origin_signed(n: u8) -> Origin {
	Origin::signed(test_public(n))
}

/// Creates a provider and delegator MSA and sets the delegation relationship.
/// # Returns
/// * (u8, u64) - Returns a delegator_msa_id and provider_msa_id.
pub fn test_create_delegator_msa_with_provider() -> (u8, u64) {
	let (key_pair, _) = sr25519::Pair::generate();
	let provider_account = key_pair.public();
	let delegator_msa_id: u8 = 1;

	let add_provider_payload =
		pallet_msa::AddProvider { authorized_msa_id: delegator_msa_id.into(), permission: 0 };
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

	let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

	assert_ok!(Msa::create(test_origin_signed(delegator_msa_id)));
	assert_ok!(Msa::create(Origin::signed(provider_account.into())));
	assert_ok!(Msa::add_provider_to_msa(
		test_origin_signed(delegator_msa_id),
		provider_account.into(),
		signature,
		add_provider_payload
	));

	let provider_msa_id: MessageSourceId = 2;
	(delegator_msa_id, provider_msa_id)
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_keystore() -> sp_io::TestExternalities {
	use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStorePtr};
	use sp_std::sync::Arc;

	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(KeyStore::new()) as SyncCryptoStorePtr));

	ext
}
