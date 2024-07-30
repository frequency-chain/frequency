use crate as pallet_handles;
pub use pallet_handles::Call as HandlesCall;
use parity_scale_codec::Decode;

use common_primitives::{
	handles::*,
	msa::{MessageSourceId, MsaLookup, MsaValidator},
	node::AccountId,
	utils::wrap_binary_data,
};
use frame_support::traits::{ConstU16, ConstU32, OnFinalize, OnInitialize};
use sp_core::{crypto::AccountId32, sr25519, ByteArray, Encode, Pair, H256};
use sp_runtime::{
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	BuildStorage, DispatchError, MultiSignature,
};

use handles_utils::converter::convert_to_canonical;
pub const INVALID_MSA_ID: MessageSourceId = 100;

type Block = frame_system::mocking::MockBlockU32<Test>;

pub struct MsaInfoHandler;

impl MsaLookup for MsaInfoHandler {
	type AccountId = AccountId;

	fn get_msa_id(key: &AccountId) -> Option<MessageSourceId> {
		if *key == test_public(INVALID_MSA_ID as u8) {
			return None
		}

		Some(MessageSourceId::decode(&mut key.as_slice()).unwrap())
	}
}

impl MsaValidator for MsaInfoHandler {
	type AccountId = AccountId;

	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError> {
		if *key == test_public(INVALID_MSA_ID as u8) {
			return Err(DispatchError::Other("some error"))
		}

		Ok(MessageSourceId::decode(&mut key.as_slice()).unwrap())
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Handles: pallet_handles::{Pallet, Call, Storage, Event<T>},
	}
);

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
	type RuntimeTask = RuntimeTask;
	type RuntimeEvent = RuntimeEvent;
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
}

impl pallet_handles::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo = ();

	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;

	/// A type that will supply MSA related information
	type MsaInfoProvider = MsaInfoHandler;

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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_keystore() -> sp_io::TestExternalities {
	use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
	use sp_std::sync::Arc;

	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));

	ext
}

// Create and return a simple test AccountId32 constructed with the desired integer.
pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

// Run to the given block number
pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Handles::on_initialize(System::block_number());
	}
}

// Create a signed claim handle payload
pub fn get_signed_claims_payload(
	account: &sr25519::Pair,
	handle: Vec<u8>,
	signature_expiration: u32,
) -> (ClaimHandlePayload<u32>, MultiSignature) {
	let base_handle = handle;
	let payload = ClaimHandlePayload::new(base_handle.clone(), signature_expiration);
	let encoded_payload = wrap_binary_data(payload.encode());
	let proof: MultiSignature = account.sign(&encoded_payload).into();

	(payload, proof)
}

/// Creates a full display handle by combining a base handle string with a suffix generated
/// from an index into the suffix sequence.
///
/// # Arguments
///
/// * `base_handle_str` - The base handle string.
/// * `suffix_sequence_index` - The index into the suffix sequence.
///
/// # Returns
///
/// * `DisplayHandle` - The full display handle.
///
pub fn create_full_handle_for_index(
	base_handle_str: &str,
	suffix_sequence_index: SequenceIndex,
) -> Vec<u8> {
	// Convert base handle into a canonical base
	let canonical_handle_str = convert_to_canonical(&base_handle_str);

	// Generate suffix from index into the suffix sequence
	let suffix = Handles::generate_suffix_for_canonical_handle(
		&canonical_handle_str,
		suffix_sequence_index as usize,
	)
	.unwrap_or_default();

	let display_handle = Handles::build_full_display_handle(base_handle_str, suffix).unwrap();
	display_handle.into_inner()
}
