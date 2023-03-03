use crate::{
	self as pallet_msa,
	mock::{create_account, generate_test_signature, new_test_ext, run_to_block},
	types::AddKeyData,
	Config, Error, PayloadSignatureRegistryRingPointer,
};

use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, EitherOfDiverse, Everything, Get},
};
use frame_system::EnsureRoot;
use pallet_collective;

use sp_core::{sr25519, Encode, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	MultiSignature,
};

pub use common_runtime::constants::*;

use common_primitives::{
	msa::SignatureRegistryPointer,
	node::{AccountId, BlockNumber},
	utils::wrap_binary_data,
};

pub use pallet_msa::Call as MsaCall;

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
		Schemas: pallet_schemas::{Pallet, Call, Storage, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>},
	}
);

// See https://paritytech.github.io/substrate/master/pallet_collective/index.html for
// the descriptions of these configs.
type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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

impl pallet_schemas::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MinSchemaModelSizeBytes = ConstU32<10>;
	type SchemaModelMaxBytesBoundedVecLimit = ConstU32<10>;
	type MaxSchemaRegistrations = ConstU16<10>;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to create schemas via governance
	type CreateSchemaViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type MaxSchemaSettingsPerSchema = ConstU32<1>;
}

parameter_types! {
	pub const MaxPublicKeysPerMsa: u8 = 10;
	pub const MaxProviderNameSize: u32 = 16;
	pub const MaxSchemas: u32 = 5;
}

parameter_types! {
	pub const MaxSchemaGrantsPerDelegation: u32 = 30;
}

impl Clone for MaxSchemaGrantsPerDelegation {
	fn clone(&self) -> Self {
		MaxSchemaGrantsPerDelegation {}
	}
}

impl Eq for MaxSchemaGrantsPerDelegation {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxSchemaGrantsPerDelegation {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxSchemaGrantsPerDelegation {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

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
		let threshold: u32 = ((Council::members().len() / 2) + 1) as u32;
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		Council::proposal_count()
	}
}

impl pallet_msa::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type ConvertIntoAccountId32 = ConvertInto;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	type MaxPublicKeysPerMsa = MaxPublicKeysPerMsa;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type MaxProviderNameSize = MaxProviderNameSize;
	type SchemaValidator = Schemas;
	type MortalityWindowSize = ConstU32<100>;
	type MaxSignaturesStored = ConstU32<20>;
	// The origin that is allowed to create providers via governance
	// It has to be this way so benchmarks will pass in CI.
	type CreateProviderViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureMembers<AccountId, CouncilCollective, 1>,
	>;
}
#[test]
pub fn cannot_register_too_many_signatures() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mortality_block: BlockNumber = 3;

		let limit: u32 = <Test as Config>::MaxSignaturesStored::get();
		for _i in 0..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
		}

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::SignatureRegistryLimitExceeded
		);
	})
}

#[test]
pub fn stores_signature_and_increments_count() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1 as u64);
		let mortality_block: BlockNumber = 51;
		let signature = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature, mortality_block.into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature.clone(),
				oldest: signature.clone(),
				count: 1,
			}),
			<PayloadSignatureRegistryRingPointer<Test>>::get()
		);

		let oldest: MultiSignature = signature.clone();

		// Expect that the newest changes
		let signature_1 = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature_1, mortality_block.into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature_1.clone(),
				oldest: signature.clone(),
				count: 2,
			}),
			<PayloadSignatureRegistryRingPointer<Test>>::get()
		);

		let mut newest: MultiSignature = signature_1.clone();

		// Fill up the registry
		let limit: u32 = <Test as Config>::MaxSignaturesStored::get();
		for _i in 2..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
			newest = sig.clone();
		}

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: newest.clone(),
				oldest: oldest.clone(),
				count: limit
			}),
			<PayloadSignatureRegistryRingPointer<Test>>::get()
		);

		run_to_block((mortality_block + 1).into());

		// Test that the next one changes the oldest signature.
		let signature_n = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature_n, (mortality_block + 10).into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature_n.clone(),
				oldest: signature_1.clone(),
				count: limit,
			}),
			<PayloadSignatureRegistryRingPointer<Test>>::get()
		);
	})
}

#[test]
pub fn clears_stale_signatures_after_mortality_limit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mortality_block: BlockNumber = 3;

		let limit: u32 = <Test as Config>::MaxSignaturesStored::get();
		for _i in 0..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
		}

		run_to_block((mortality_block).into());

		// Cannot do it yet as we are at the mortality_block

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, (mortality_block + 10).into()),
			Error::<Test>::SignatureRegistryLimitExceeded
		);

		run_to_block((mortality_block + 1).into());

		// Now it is OK as we are +1 past the mortality_block
		assert_ok!(Msa::register_signature(sig1, (mortality_block + 10).into()));
	})
}

#[test]
pub fn cannot_register_signature_with_mortality_out_of_bounds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mut mortality_block: BlockNumber = 11_323;

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofNotYetValid
		);

		mortality_block = 11_122;
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofHasExpired
		);
	})
}

struct TestCase {
	current: u64,
	mortality: u64,
	run_to: u64,
	expected_ok: bool,
}

#[test]
pub fn add_msa_key_replay_fails() {
	new_test_ext().execute_with(|| {
		// these should all fail replay
		let test_cases: Vec<TestCase> = vec![
			TestCase {
				current: 10_949u64,
				mortality: 11_001u64,
				run_to: 10_848u64,
				expected_ok: true,
			},
			TestCase { current: 1u64, mortality: 3u64, run_to: 5u64, expected_ok: false },
			TestCase { current: 99u64, mortality: 101u64, run_to: 100u64, expected_ok: true },
			TestCase {
				current: 1_100u64,
				mortality: 1_199u64,
				run_to: 1_198u64,
				expected_ok: true,
			},
			TestCase {
				current: 1_102u64,
				mortality: 1_201u64,
				run_to: 1_200u64,
				expected_ok: true,
			},
			TestCase {
				current: 1_099u64,
				mortality: 1_148u64,
				run_to: 1_101u64,
				expected_ok: true,
			},
			TestCase {
				current: 1_000_000u64,
				mortality: 1_000_000u64,
				run_to: 1_000_000u64,
				expected_ok: false,
			},
		];

		let (new_msa_id, key_pair_provider) = create_account();
		let account_provider = key_pair_provider.public();
		for tc in test_cases {
			System::set_block_number(tc.current);

			let (new_key_pair, _) = sr25519::Pair::generate();

			let add_new_key_data = AddKeyData {
				msa_id: new_msa_id,
				expiration: tc.mortality,
				new_public_key: new_key_pair.public().into(),
			};

			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

			let signature_owner: MultiSignature =
				key_pair_provider.sign(&encode_data_new_key_data).into();

			let signature_new_key: MultiSignature =
				new_key_pair.sign(&encode_data_new_key_data).into();

			run_to_block(tc.run_to);

			let add_key_response: bool = Msa::add_public_key_to_msa(
				RuntimeOrigin::signed(account_provider.into()),
				account_provider.into(),
				signature_owner.clone(),
				signature_new_key,
				add_new_key_data.clone(),
			)
			.is_ok();

			assert_eq!(add_key_response, tc.expected_ok);
		}
	})
}
