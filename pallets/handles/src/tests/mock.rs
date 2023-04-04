use crate as pallet_handles;
use codec::Decode;
pub use pallet_handles::Call as HandlesCall;

use common_primitives::{
	handles::*,
	msa::{MessageSourceId, MsaLookup, MsaValidator},
	node::AccountId,
	utils::wrap_binary_data,
};
use frame_support::{
	dispatch::DispatchError,
	traits::{ConstU16, ConstU32, ConstU64},
};
use sp_core::{crypto::AccountId32, sr25519, ByteArray, Encode, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	MultiSignature,
};

pub const INVALID_MSA_ID: MessageSourceId = 100;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub struct MsaInfoHandler;

impl MsaLookup for MsaInfoHandler {
	type AccountId = AccountId;

	fn get_msa_id(key: &AccountId) -> Option<MessageSourceId> {
		log::debug!("get_msa_id()");

		if *key == test_public(INVALID_MSA_ID as u8) {
			return None
		}

		Some(MessageSourceId::decode(&mut key.as_slice()).unwrap())
	}
}

impl MsaValidator for MsaInfoHandler {
	type AccountId = AccountId;

	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError> {
		log::debug!("ensure_valid_msa_key()");

		if *key == test_public(INVALID_MSA_ID as u8) {
			return Err(DispatchError::Other("some error"))
		}

		Ok(MessageSourceId::decode(&mut key.as_slice()).unwrap())
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
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

impl pallet_handles::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo = ();

	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;

	/// A type that will supply MSA related information
	type MsaInfoProvider = MsaInfoHandler;

	/// The minimum suffix value
	type HandleSuffixMin = ConstU32<10>;

	/// The maximum suffix value
	type HandleSuffixMax = ConstU32<99>;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

// Create and return a simple test AccountId32 constructed with the desired integer.
pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

// Create a signed claims payload
pub fn get_signed_claims_payload(
	account: &sr25519::Pair,
	handle: Vec<u8>,
) -> (ClaimHandlePayload, MultiSignature) {
	let base_handle = handle;
	let payload = ClaimHandlePayload::new(base_handle.clone());
	let encoded_payload = wrap_binary_data(payload.encode());
	let proof: MultiSignature = account.sign(&encoded_payload).into();

	(payload, proof)
}
