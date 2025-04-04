use crate::{self as pallet_msa, types::EMPTY_FUNCTION, AddProvider};
use common_primitives::{
	msa::MessageSourceId, node::BlockNumber, schema::SchemaId, utils::wrap_binary_data,
};
use common_runtime::constants::DAYS;
use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU16, ConstU32, EitherOfDiverse, OnFinalize, OnInitialize},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_collective::{self, Members};
use parity_scale_codec::MaxEncodedLen;
use sp_core::{
	offchain::{testing, testing::OffchainState, OffchainDbExt, OffchainWorkerExt},
	sr25519,
	sr25519::Public,
	Encode, Pair, H256,
};
use sp_runtime::{
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	AccountId32, BuildStorage, DispatchError, MultiSignature,
};
extern crate alloc;
use alloc::sync::Arc;

pub use pallet_msa::Call as MsaCall;

#[cfg(feature = "runtime-benchmarks")]
use pallet_collective::ProposalCount;

use common_primitives::node::AccountId;

type Block = frame_system::mocking::MockBlockU32<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>},
		Schemas: pallet_schemas::{Pallet, Call, Storage, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>},
		Handles: pallet_handles::{Pallet, Call, Storage, Event<T>},
	}
);

// See https://paritytech.github.io/substrate/master/pallet_collective/index.html for
// the descriptions of these configs.
parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::MAX);
	pub MaxProposalWeight: frame_support::weights::Weight  = sp_runtime::Perbill::from_percent(50) * BlockWeights::get().max_block;
	pub const SchemaModelMaxBytesBoundedVecLimit: u32 = 10;
}

impl Encode for SchemaModelMaxBytesBoundedVecLimit {}

impl MaxEncodedLen for SchemaModelMaxBytesBoundedVecLimit {
	fn max_encoded_len() -> usize {
		u32::max_encoded_len()
	}
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = ConstU32<{ 5 * DAYS }>;
	type MaxProposals = ConstU32<25>;
	type MaxMembers = ConstU32<10>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId32>;
	type MaxProposalWeight = MaxProposalWeight;
	type DisapproveOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeTask = RuntimeTask;
	type BlockHashCount = ConstU32<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

impl pallet_schemas::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MinSchemaModelSizeBytes = ConstU32<10>;
	type SchemaModelMaxBytesBoundedVecLimit = SchemaModelMaxBytesBoundedVecLimit;
	type MaxSchemaRegistrations = ConstU16<10>;
	type MaxSchemaSettingsPerSchema = ConstU32<1>;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to create schemas via governance
	// It has to be this way so benchmarks will pass in CI.
	type CreateSchemaViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
}

impl pallet_handles::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo = ();

	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;

	/// A type that will supply MSA related information
	type MsaInfoProvider = Msa;

	/// The minimum suffix value
	type HandleSuffixMin = ConstU16<10>;

	/// The maximum suffix value
	type HandleSuffixMax = ConstU16<99>;

	/// The mortality window for a handle claim
	type MortalityWindowSize = ConstU32<150>;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = ();
}

// Needs parameter_types! for the Option and statics
parameter_types! {
	pub static MaxPublicKeysPerMsa: u8 = 255;
	pub static MaxSignaturesStored: Option<u32> = Some(8000);
}
pub type MaxProviderNameSize = ConstU32<16>;
pub type MaxSchemaGrantsPerDelegation = ConstU32<30>;

/// Interface to collective pallet to propose a proposal.
pub struct CouncilProposalProvider;

impl pallet_msa::ProposalProvider<AccountId, RuntimeCall> for CouncilProposalProvider {
	fn propose(
		who: AccountId,
		threshold: u32,
		proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	fn propose_with_simple_majority(
		who: AccountId,
		proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		let members = Members::<Test, CouncilCollective>::get();
		let threshold: u32 = ((members.len() / 2) + 1) as u32;
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		ProposalCount::<Test, CouncilCollective>::get()
	}
}

impl pallet_msa::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type ConvertIntoAccountId32 = ConvertInto;
	type MaxPublicKeysPerMsa = MaxPublicKeysPerMsa;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type MaxProviderNameSize = MaxProviderNameSize;
	type SchemaValidator = Schemas;
	type HandleProvider = Handles;
	type MortalityWindowSize = ConstU32<100>;
	type MaxSignaturesStored = MaxSignaturesStored;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to create providers via governance
	// It has to be this way so benchmarks will pass in CI.
	type CreateProviderViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureMembers<AccountId, CouncilCollective, 1>,
	>;
}

pub fn set_max_signature_stored(max: u32) {
	MAX_SIGNATURES_STORED.with(|v| *v.borrow_mut() = Some(max));
}

pub fn set_max_public_keys_per_msa(max: u8) {
	MAX_PUBLIC_KEYS_PER_MSA.with(|v| *v.borrow_mut() = max);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	set_max_signature_stored(8000);
	set_max_public_keys_per_msa(255);
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn new_test_with_offchain_ext(
) -> (sp_io::TestExternalities, Arc<parking_lot::RwLock<OffchainState>>) {
	set_max_signature_stored(8000);
	set_max_public_keys_per_msa(255);
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	let (offchain, state) = testing::TestOffchainExt::with_offchain_db(ext.offchain_db());
	ext.register_extension(OffchainDbExt::new(offchain.clone()));
	ext.register_extension(OffchainWorkerExt::new(offchain));
	ext.execute_with(|| System::set_block_number(1));
	(ext, state)
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Msa::on_initialize(System::block_number());
	}
}

/// Create and return a simple test AccountId32 constructed with the desired integer.
pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

/// Create and return a simple signed origin from a test_public constructed with the desired integer,
/// for passing to an extrinsic call
pub fn test_origin_signed(n: u8) -> RuntimeOrigin {
	RuntimeOrigin::signed(test_public(n))
}

/// Create a new keypair and an MSA associated with its public key.
/// # Returns
/// (MessageSourceId, Pair) - a tuple with the MSA and the new Account key pair
pub fn create_account() -> (MessageSourceId, sr25519::Pair) {
	let (key_pair, _) = sr25519::Pair::generate();
	let result_key = Msa::create_account(AccountId32::from(key_pair.public()), EMPTY_FUNCTION);
	assert_ok!(&result_key);
	let (msa_id, _) = result_key.unwrap();
	(msa_id, key_pair)
}

/// Creates and signs an `AddProvider` struct using the provided delegator keypair and provider MSA
/// # Returns
/// (MultiSignature, AddProvider) - Returns a tuple with the signature and the AddProvider struct
pub fn create_and_sign_add_provider_payload(
	delegator_pair: sr25519::Pair,
	provider_msa: MessageSourceId,
) -> (MultiSignature, AddProvider) {
	create_and_sign_add_provider_payload_with_schemas(delegator_pair, provider_msa, None, 10)
}

/// Creates and signs an `AddProvider` struct using the provided delegator keypair, provider MSA and schema ids
/// # Returns
/// (MultiSignature, AddProvider) - Returns a tuple with the signature and the AddProvider struct
pub fn create_and_sign_add_provider_payload_with_schemas(
	delegator_pair: sr25519::Pair,
	provider_msa: MessageSourceId,
	schema_ids: Option<Vec<SchemaId>>,
	expiration: BlockNumber,
) -> (MultiSignature, AddProvider) {
	let add_provider_payload = AddProvider::new(provider_msa, schema_ids, expiration);
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
	let signature: MultiSignature = delegator_pair.sign(&encode_add_provider_data).into();
	(signature, add_provider_payload)
}

/// Creates a provider and delegator MSA and sets the delegation relationship.
// create and set up delegations for a delegator and provider, but for convenience only
/// # Returns
/// * (u8, Public) - Returns a provider_msa_id and a delegator account.
pub fn create_provider_msa_and_delegator() -> (u64, Public) {
	let (provider_msa_id, _, _, delegator_account) = create_provider_delegator_msas();
	(provider_msa_id, delegator_account)
}

// create and set up delegations for a delegator and provider, but for convenience only
// return delegator msa and provider account for testing delegator-submitted extrinsics
/// # Returns
/// * (u8, Public) - Returns a delegator_msa_id and a provider_account.
pub fn create_delegator_msa_and_provider() -> (u64, Public) {
	let (_, provider_account, delegator_msa_id, _) = create_provider_delegator_msas();
	(delegator_msa_id, provider_account)
}

// create and set up delegations for a delegator and provider and return it all
pub fn create_provider_delegator_msas() -> (u64, Public, u64, Public) {
	let (provider_msa_id, provider_pair) = create_account();
	let provider_account = provider_pair.public();

	let (delegator_msa_id, delegator_pair) = create_account();
	let delegator_account = delegator_pair.public();

	let (delegator_signature, add_provider_payload) =
		create_and_sign_add_provider_payload(delegator_pair, provider_msa_id);

	// Register provider
	assert_ok!(Msa::create_provider(
		RuntimeOrigin::signed(provider_account.into()),
		Vec::from("Foo")
	));

	assert_ok!(Msa::grant_delegation(
		RuntimeOrigin::signed(provider_account.into()),
		delegator_account.into(),
		delegator_signature,
		add_provider_payload
	));
	(provider_msa_id, provider_account, delegator_msa_id, delegator_account)
}

// Create a provider with given name
pub fn create_provider_with_name(name: &str) -> (u64, Public) {
	let (provider_msa_id, provider_pair) = create_account();
	let provider_account = provider_pair.public();
	// Register provider
	assert_ok!(Msa::create_provider(
		RuntimeOrigin::signed(provider_account.into()),
		Vec::from(name)
	));
	(provider_msa_id, provider_account)
}

pub fn generate_test_signature() -> MultiSignature {
	let (key_pair, _) = sr25519::Pair::generate();
	let fake_data = H256::random();
	key_pair.sign(fake_data.as_bytes()).into()
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_keystore() -> sp_io::TestExternalities {
	use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));

	ext
}
