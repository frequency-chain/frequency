use crate as pallet_messages;
use common_primitives::{
	msa::{
		Delegation, DelegationValidator, DelegatorId, MessageSourceId, MsaLookup, MsaValidator,
		ProviderId, ProviderLookup, SchemaGrantValidator,
	},
	schema::*,
};

use codec::{Encode, MaxEncodedLen};
use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	traits::{ConstU16, ConstU32, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, DispatchError,
};
use std::fmt::Formatter;

type Block = frame_system::mocking::MockBlockU32<Test>;

pub const INVALID_SCHEMA_ID: SchemaId = 65534;
pub const IPFS_SCHEMA_ID: SchemaId = 50;

pub const IPFS_PAYLOAD_LENGTH: u32 = 1200;

pub const DUMMY_CID_BASE32: &[u8; 59] =
	b"bafkreieb2x7yyuhy6hmct4j7tkmgnthrfpqyo4mt5nscx7pvc6oiweiwjq";
pub const DUMMY_CID_BASE64: &[u8; 49] = b"mAVUSIIHV/4xQ+PHYKfE/mphmzPEr4Ydxk+tkK/31F5yLERZM";

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		MessagesPallet: pallet_messages::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
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
	type MaxConsumers = ConstU32<16>;
}

pub type MaxMessagesPerBlock = ConstU32<500>;
pub type MaxSchemaGrantsPerDelegation = ConstU32<30>;

// Needs parameter_types! for the impls below
parameter_types! {
	// Max payload size was picked specifically to be large enough to accommodate
	// a CIDv1 using SHA2-256, but too small to accommodate CIDv1 w/SHA2-512.
	// This is purely so that we can test the error condition. Real world configuration
	// should have this set large enough to accommodate the largest possible CID.
	// Take care when adding new tests for on-chain (not IPFS) messages that the payload
	// is not too big.
	pub const MessagesMaxPayloadSizeBytes: u32 = 73;
}

impl std::fmt::Debug for MessagesMaxPayloadSizeBytes {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MessagesMaxPayloadSizeBytes")
			.field("v", &MessagesMaxPayloadSizeBytes::get())
			.finish()
	}
}

impl PartialEq for MessagesMaxPayloadSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Clone for MessagesMaxPayloadSizeBytes {
	fn clone(&self) -> Self {
		MessagesMaxPayloadSizeBytes {}
	}
}

impl Encode for MessagesMaxPayloadSizeBytes {}

impl MaxEncodedLen for MessagesMaxPayloadSizeBytes {
	fn max_encoded_len() -> usize {
		MessagesMaxPayloadSizeBytes::get() as usize
	}
}

pub struct MsaInfoHandler;
pub struct DelegationInfoHandler;
pub struct SchemaGrantValidationHandler;
impl MsaLookup for MsaInfoHandler {
	type AccountId = u64;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId> {
		if *key == 1000 {
			return None
		}
		if *key == 2000 {
			return Some(2000 as MessageSourceId)
		}
		Some(get_msa_from_account(*key) as MessageSourceId)
	}
}

impl MsaValidator for MsaInfoHandler {
	type AccountId = u64;

	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError> {
		if *key == 1000 {
			return Err(DispatchError::Other("some error"))
		}
		if *key == 2000 {
			return Ok(2000)
		}

		Ok(get_msa_from_account(*key))
	}
}
impl ProviderLookup for DelegationInfoHandler {
	type BlockNumber = u32;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type SchemaId = SchemaId;

	fn get_delegation_of(
		_delegator: DelegatorId,
		provider: ProviderId,
	) -> Option<Delegation<SchemaId, Self::BlockNumber, MaxSchemaGrantsPerDelegation>> {
		if provider == ProviderId(2000) {
			return None
		};
		Some(Delegation { revoked_at: 100, schema_permissions: Default::default() })
	}
}
impl DelegationValidator for DelegationInfoHandler {
	type BlockNumber = u32;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type SchemaId = SchemaId;

	fn ensure_valid_delegation(
		provider: ProviderId,
		_delegator: DelegatorId,
		_block_number: Option<Self::BlockNumber>,
	) -> Result<
		Delegation<SchemaId, Self::BlockNumber, Self::MaxSchemaGrantsPerDelegation>,
		DispatchError,
	> {
		if provider == ProviderId(2000) {
			return Err(DispatchError::Other("some delegation error"))
		};

		Ok(Delegation { schema_permissions: Default::default(), revoked_at: Default::default() })
	}
}
impl<BlockNumber> SchemaGrantValidator<BlockNumber> for SchemaGrantValidationHandler {
	fn ensure_valid_schema_grant(
		provider: ProviderId,
		delegator: DelegatorId,
		_schema_id: SchemaId,
		_block_number: BlockNumber,
	) -> DispatchResult {
		match DelegationInfoHandler::get_delegation_of(delegator, provider) {
			Some(_) => Ok(()),
			None => Err(DispatchError::Other("no schema grant or delegation")),
		}
	}
}

pub struct SchemaHandler;
impl SchemaProvider<u16> for SchemaHandler {
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		if schema_id == INVALID_SCHEMA_ID {
			return None
		}
		if schema_id == IPFS_SCHEMA_ID {
			return Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::Parquet,
				payload_location: PayloadLocation::IPFS,
				settings: Vec::new(),
			})
		}

		Some(SchemaResponse {
			schema_id,
			model: r#"schema"#.to_string().as_bytes().to_vec(),
			model_type: ModelType::AvroBinary,
			payload_location: PayloadLocation::OnChain,
			settings: Vec::new(),
		})
	}
}

impl pallet_messages::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MsaInfoProvider = MsaInfoHandler;
	type SchemaGrantValidator = SchemaGrantValidationHandler;
	type SchemaProvider = SchemaHandler;
	type WeightInfo = ();
	type MaxMessagesPerBlock = MaxMessagesPerBlock;
	type MessagesMaxPayloadSizeBytes = MessagesMaxPayloadSizeBytes;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = ();
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			MessagesPallet::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		MessagesPallet::on_initialize(System::block_number());
	}
}

pub fn get_msa_from_account(account_id: u64) -> u64 {
	account_id + 100
}
