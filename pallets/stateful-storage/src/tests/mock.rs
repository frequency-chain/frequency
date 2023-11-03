use crate as pallet_stateful_storage;
use parity_scale_codec::Decode;

use crate::test_common::{
	constants,
	constants::{BENCHMARK_SIGNATURE_ACCOUNT_SEED, SIGNATURE_MSA_ID},
};
use common_primitives::{
	msa::{
		Delegation, DelegationValidator, DelegatorId, MessageSourceId, MsaLookup, MsaValidator,
		ProviderId, ProviderLookup, SchemaGrantValidator,
	},
	node::AccountId,
	schema::{
		ModelType, PayloadLocation, SchemaId, SchemaInfoResponse, SchemaProvider, SchemaResponse,
		SchemaSetting,
	},
};
use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	traits::{ConstU16, ConstU32},
	Twox128,
};
use frame_system as system;
use sp_core::{crypto::AccountId32, sr25519, ByteArray, Pair, H256};
use sp_runtime::{
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
	BuildStorage, DispatchError,
};

type Block = frame_system::mocking::MockBlockU32<Test>;

pub const INVALID_SCHEMA_ID: SchemaId = SchemaId::MAX;
pub const INVALID_MSA_ID: MessageSourceId = 100;
pub const TEST_ACCOUNT_SEED: [u8; 32] = [0; 32];

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		StatefulStoragePallet: pallet_stateful_storage::{Pallet, Call, Storage, Event<T>},
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
	type AccountId = AccountId;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type MaxItemizedActionsCount = ConstU32<6>;
pub type MaxSchemaGrantsPerDelegation = ConstU32<30>;
pub type StatefulMortalityWindowSize = ConstU32<10>;

// Needs parameter_types! for the impls below
parameter_types! {
	pub const MaxItemizedPageSizeBytes: u32 = 1024;
	pub const MaxItemizedBlobSizeBytes: u32 = 64;
	pub const MaxPaginatedPageSizeBytes: u32 = 1024;
	pub const MaxPaginatedPageId: u16 = 32;
}

impl Default for MaxItemizedPageSizeBytes {
	fn default() -> Self {
		Self
	}
}

impl Default for MaxPaginatedPageSizeBytes {
	fn default() -> Self {
		Self
	}
}

pub struct MsaInfoHandler;
pub struct DelegationInfoHandler;
pub struct SchemaGrantValidationHandler;
impl MsaLookup for MsaInfoHandler {
	type AccountId = AccountId;

	fn get_msa_id(key: &AccountId) -> Option<MessageSourceId> {
		if *key == test_public(INVALID_MSA_ID) ||
			*key == get_invalid_msa_signature_account().public().into()
		{
			return None
		}

		if *key == get_signature_benchmarks_public_account().into() ||
			*key == get_signature_account().1.public().into()
		{
			return Some(constants::SIGNATURE_MSA_ID)
		}

		Some(MessageSourceId::decode(&mut key.as_slice()).unwrap())
	}
}

impl MsaValidator for MsaInfoHandler {
	type AccountId = AccountId;

	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError> {
		if *key == test_public(INVALID_MSA_ID) ||
			*key == get_invalid_msa_signature_account().public().into()
		{
			return Err(DispatchError::Other("some error"))
		}

		if *key == get_signature_benchmarks_public_account().into() ||
			*key == get_signature_account().1.public().into()
		{
			return Ok(constants::SIGNATURE_MSA_ID)
		}

		Ok(MessageSourceId::decode(&mut key.as_slice()).unwrap())
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
		schema_id: SchemaId,
		_block_number: BlockNumber,
	) -> DispatchResult {
		if schema_id == constants::UNDELEGATED_PAGINATED_SCHEMA ||
			schema_id == constants::UNDELEGATED_ITEMIZED_APPEND_ONLY_SCHEMA ||
			schema_id == constants::UNDELEGATED_ITEMIZED_SCHEMA
		{
			return Err(DispatchError::Other("no schema grant or delegation"))
		}

		match DelegationInfoHandler::get_delegation_of(delegator, provider) {
			Some(_) => Ok(()),
			None => Err(DispatchError::Other("no schema grant or delegation")),
		}
	}
}

pub struct SchemaHandler;
impl SchemaProvider<u16> for SchemaHandler {
	// For testing/benchmarking. Zero value returns None, Odd for Itemized, Even for Paginated
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		match schema_id {
			constants::ITEMIZED_SCHEMA | constants::UNDELEGATED_ITEMIZED_SCHEMA =>
				Some(SchemaResponse {
					schema_id,
					model: r#"schema"#.to_string().as_bytes().to_vec(),
					model_type: ModelType::AvroBinary,
					payload_location: PayloadLocation::Itemized,
					settings: Vec::new(),
				}),
			constants::ITEMIZED_APPEND_ONLY_SCHEMA |
			constants::UNDELEGATED_ITEMIZED_APPEND_ONLY_SCHEMA => Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::AvroBinary,
				payload_location: PayloadLocation::Itemized,
				settings: Vec::try_from(vec![SchemaSetting::AppendOnly]).unwrap(),
			}),
			constants::ITEMIZED_SIGNATURE_REQUIRED_SCHEMA => Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::AvroBinary,
				payload_location: PayloadLocation::Itemized,
				settings: Vec::try_from(vec![SchemaSetting::SignatureRequired]).unwrap(),
			}),
			constants::PAGINATED_SCHEMA | constants::UNDELEGATED_PAGINATED_SCHEMA =>
				Some(SchemaResponse {
					schema_id,
					model: r#"schema"#.to_string().as_bytes().to_vec(),
					model_type: ModelType::AvroBinary,
					payload_location: PayloadLocation::Paginated,
					settings: Vec::new(),
				}),
			constants::PAGINATED_SIGNED_SCHEMA => Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::AvroBinary,
				payload_location: PayloadLocation::Paginated,
				settings: Vec::try_from(vec![SchemaSetting::SignatureRequired]).unwrap(),
			}),
			constants::PAGINATED_APPEND_ONLY_SCHEMA => Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::AvroBinary,
				payload_location: PayloadLocation::Paginated,
				settings: Vec::try_from(vec![SchemaSetting::AppendOnly]).unwrap(),
			}),
			INVALID_SCHEMA_ID => None,

			_ => Some(SchemaResponse {
				schema_id,
				model: r#"schema"#.to_string().as_bytes().to_vec(),
				model_type: ModelType::AvroBinary,
				payload_location: PayloadLocation::OnChain,
				settings: Vec::from(vec![SchemaSetting::AppendOnly]),
			}),
		}
	}

	fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse> {
		Self::get_schema_by_id(schema_id).and_then(|schema| {
			Some(SchemaInfoResponse {
				schema_id: schema.schema_id,
				settings: schema.settings,
				model_type: schema.model_type,
				payload_location: schema.payload_location,
			})
		})
	}
}

impl Clone for MaxPaginatedPageId {
	fn clone(&self) -> Self {
		MaxPaginatedPageId {}
	}
}

impl Eq for MaxPaginatedPageId {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxPaginatedPageId {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxPaginatedPageId {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxItemizedPageSizeBytes {
	fn clone(&self) -> Self {
		MaxItemizedPageSizeBytes {}
	}
}

impl Eq for MaxItemizedPageSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxItemizedPageSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxItemizedPageSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxPaginatedPageSizeBytes {
	fn clone(&self) -> Self {
		MaxPaginatedPageSizeBytes {}
	}
}

impl Eq for MaxPaginatedPageSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxPaginatedPageSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxPaginatedPageSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxItemizedBlobSizeBytes {
	fn clone(&self) -> Self {
		MaxItemizedBlobSizeBytes {}
	}
}

impl Eq for MaxItemizedBlobSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxItemizedBlobSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxItemizedBlobSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl pallet_stateful_storage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MsaInfoProvider = MsaInfoHandler;
	type SchemaGrantValidator = SchemaGrantValidationHandler;
	type SchemaProvider = SchemaHandler;
	type WeightInfo = ();
	type MaxItemizedBlobSizeBytes = MaxItemizedBlobSizeBytes;
	type MaxPaginatedPageId = MaxPaginatedPageId;
	type MaxItemizedPageSizeBytes = MaxItemizedPageSizeBytes;
	type MaxPaginatedPageSizeBytes = MaxPaginatedPageSizeBytes;
	type MaxItemizedActionsCount = MaxItemizedActionsCount;
	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = ();
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = ();
	type KeyHasher = Twox128;
	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;
	/// The number of blocks per virtual bucket
	type MortalityWindowSize = StatefulMortalityWindowSize;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn get_signature_account() -> (MessageSourceId, sr25519::Pair) {
	(SIGNATURE_MSA_ID, sr25519::Pair::from_seed_slice(TEST_ACCOUNT_SEED.as_slice()).unwrap())
}

pub fn get_invalid_msa_signature_account() -> sr25519::Pair {
	sr25519::Pair::from_seed_slice([1; 32].as_slice()).unwrap()
}

fn get_signature_benchmarks_public_account() -> sr25519::Public {
	sr25519::Pair::from_string(BENCHMARK_SIGNATURE_ACCOUNT_SEED, None)
		.unwrap()
		.public()
}

pub fn test_public(n: MessageSourceId) -> AccountId32 {
	AccountId32::new([n as u8; 32])
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
