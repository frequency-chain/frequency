//! Fixes the Weight values for Capacity transactions as static values
//! Any change in actual weight does not adjust the cost, but will still adjust the block space
//!

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(
	rustdoc::all,
	missing_docs,
	unused_parens,
	unused_imports
)]

use frame_support::{traits::Get, weights::{Weight, constants::WEIGHT_REF_TIME_PER_NANOS}};
use core::marker::PhantomData;
use common_runtime::weights::rocksdb_child_trie_weights::constants::RocksDbWeightChild;

/// The base fee for extrinsics is calculated by running benchmarks.
/// Capacity needs the base fee to remain stable and not change when benchmarks are run.
/// CAPACITY_EXTRINSIC_BASE_WEIGHT is a snapshot of the ExtrinsicBaseWeight
/// taken from: runtime/common/src/weights/extrinsic_weights.rs
///
/// Time to execute a NO-OP extrinsic, for example `System::remark`.
/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
///
/// Stats nanoseconds:
///   Min, Max: 104_713, 111_324
///   Average:  105_455
///   Median:   105_091
///   Std-Dev:  1133.64
///
/// Percentiles nanoseconds:
///   99th: 110_219
///   95th: 106_592
///   75th: 105_471
pub const CAPACITY_EXTRINSIC_BASE_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(105_455), 0);

/// Weight functions needed for pallet_msa.
pub trait WeightInfo {
	// MSA
	fn create_sponsored_account_with_delegation(s: u32) -> Weight;
	fn add_public_key_to_msa() -> Weight;
	fn grant_delegation(s: u32) -> Weight;
	fn add_recovery_commitment() -> Weight;
	fn recover_account() -> Weight;
	// Messages
	fn add_onchain_message(n: u32) -> Weight;
	fn add_ipfs_message() -> Weight;
	// Stateful-storage
	fn apply_item_actions(n: u32) -> Weight;
	fn upsert_page(s: u32) -> Weight;
	fn delete_page() -> Weight;
	fn apply_item_actions_with_signature(s: u32) -> Weight;
	fn upsert_page_with_signature(s: u32) -> Weight;
	fn delete_page_with_signature() -> Weight;
	// Handles
	fn claim_handle(b: u32) -> Weight;
	fn change_handle(b: u32) -> Weight;
}

// Update test as well to ensure static weight values `tests/stable_weights_test.rs`

// Updated to match v1.7.4 released weights

/// Weights for pallet_msa using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Msa::PayloadSignatureRegistryList` (r:2 w:2)
	/// Proof: `Msa::PayloadSignatureRegistryList` (`max_values`: Some(50000), `max_size`: Some(144), added: 2124, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryPointer` (r:1 w:1)
	/// Proof: `Msa::PayloadSignatureRegistryPointer` (`max_values`: Some(1), `max_size`: Some(140), added: 635, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:2 w:1)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::ProviderToRegistryEntryV2` (r:1 w:0)
	/// Proof: `Msa::ProviderToRegistryEntryV2` (`max_values`: None, `max_size`: Some(3762), added: 6237, mode: `MaxEncodedLen`)
	/// Storage: `Msa::CurrentMsaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Msa::CurrentMsaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyCountForMsaId` (r:1 w:1)
	/// Proof: `Msa::PublicKeyCountForMsaId` (`max_values`: None, `max_size`: Some(17), added: 2492, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:1)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:0)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[0, 30]`.
	fn create_sponsored_account_with_delegation(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1260`
		//  Estimated: `7722`
		// Minimum execution time: 150_945_000 picoseconds.
		Weight::from_parts(158_199_500, 7722)
			// Standard Error: 44_798
			.saturating_add(Weight::from_parts(53_126, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(10_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Msa::PayloadSignatureRegistryList` (r:4 w:4)
	/// Proof: `Msa::PayloadSignatureRegistryList` (`max_values`: Some(50000), `max_size`: Some(144), added: 2124, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryPointer` (r:1 w:1)
	/// Proof: `Msa::PayloadSignatureRegistryPointer` (`max_values`: Some(1), `max_size`: Some(140), added: 635, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:2 w:1)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyCountForMsaId` (r:1 w:1)
	/// Proof: `Msa::PublicKeyCountForMsaId` (`max_values`: None, `max_size`: Some(17), added: 2492, mode: `MaxEncodedLen`)
	fn add_public_key_to_msa() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1536`
		//  Estimated: `9981`
		// Minimum execution time: 231_148_000 picoseconds.
		Weight::from_parts(237_413_000, 9981)
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Msa::PayloadSignatureRegistryList` (r:2 w:2)
	/// Proof: `Msa::PayloadSignatureRegistryList` (`max_values`: Some(50000), `max_size`: Some(144), added: 2124, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryPointer` (r:1 w:1)
	/// Proof: `Msa::PayloadSignatureRegistryPointer` (`max_values`: Some(1), `max_size`: Some(140), added: 635, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:2 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::ProviderToRegistryEntryV2` (r:1 w:0)
	/// Proof: `Msa::ProviderToRegistryEntryV2` (`max_values`: None, `max_size`: Some(3762), added: 6237, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:1)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:0)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[0, 30]`.
	fn grant_delegation(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1310`
		//  Estimated: `7722`
		// Minimum execution time: 141_265_000 picoseconds.
		Weight::from_parts(146_783_884, 7722)
			// Standard Error: 25_530
			.saturating_add(Weight::from_parts(138_064, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Msa::PayloadSignatureRegistryList` (r:2 w:2)
	/// Proof: `Msa::PayloadSignatureRegistryList` (`max_values`: Some(50000), `max_size`: Some(144), added: 2124, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryPointer` (r:1 w:1)
	/// Proof: `Msa::PayloadSignatureRegistryPointer` (`max_values`: Some(1), `max_size`: Some(140), added: 635, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::MsaIdToRecoveryCommitment` (r:0 w:1)
	/// Proof: `Msa::MsaIdToRecoveryCommitment` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn add_recovery_commitment() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1079`
		//  Estimated: `5733`
		// Minimum execution time: 120_850_000 picoseconds.
		Weight::from_parts(124_481_000, 5733)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:2 w:1)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::RecoveryProviders` (r:1 w:0)
	/// Proof: `Msa::RecoveryProviders` (`max_values`: None, `max_size`: Some(17), added: 2492, mode: `MaxEncodedLen`)
	/// Storage: `Msa::MsaIdToRecoveryCommitment` (r:1 w:1)
	/// Proof: `Msa::MsaIdToRecoveryCommitment` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryList` (r:2 w:2)
	/// Proof: `Msa::PayloadSignatureRegistryList` (`max_values`: Some(50000), `max_size`: Some(144), added: 2124, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PayloadSignatureRegistryPointer` (r:1 w:1)
	/// Proof: `Msa::PayloadSignatureRegistryPointer` (`max_values`: Some(1), `max_size`: Some(140), added: 635, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyCountForMsaId` (r:1 w:1)
	/// Proof: `Msa::PublicKeyCountForMsaId` (`max_values`: None, `max_size`: Some(17), added: 2492, mode: `MaxEncodedLen`)
	fn recover_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1239`
		//  Estimated: `6531`
		// Minimum execution time: 152_322_000 picoseconds.
		Weight::from_parts(154_850_000, 6531)
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:0)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Messages::MessagesV2` (r:0 w:1)
	/// Proof: `Messages::MessagesV2` (`max_values`: None, `max_size`: Some(3123), added: 5598, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 3071]`.
	fn add_onchain_message(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `881`
		//  Estimated: `4177`
		// Minimum execution time: 40_860_000 picoseconds.
		Weight::from_parts(42_194_321, 4177)
			// Standard Error: 145
			.saturating_add(Weight::from_parts(959, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Messages::MessagesV2` (r:0 w:1)
	/// Proof: `Messages::MessagesV2` (`max_values`: None, `max_size`: Some(3123), added: 5598, mode: `MaxEncodedLen`)
	fn add_ipfs_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `814`
		//  Estimated: `4008`
		// Minimum execution time: 31_646_000 picoseconds.
		Weight::from_parts(32_704_000, 4008)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:0)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// The range of component `s` is `[1024, 5120]`.
	fn apply_item_actions(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `632`
		//  Estimated: `6077`
		// Minimum execution time: 37_479_000 picoseconds.
		Weight::from_parts(37_727_324, 6077)
			// Standard Error: 123
			.saturating_add(Weight::from_parts(1_075, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:0)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// The range of component `s` is `[1, 1024]`.
	fn upsert_page(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1685`
		//  Estimated: `7130`
		// Minimum execution time: 38_000_000 picoseconds.
		Weight::from_parts(39_664_500, 7130)
			// Standard Error: 199
			.saturating_add(Weight::from_parts(101, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Msa::DelegatorAndProviderToDelegation` (r:1 w:0)
	/// Proof: `Msa::DelegatorAndProviderToDelegation` (`max_values`: None, `max_size`: Some(217), added: 2692, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	fn delete_page() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1683`
		//  Estimated: `7128`
		// Minimum execution time: 38_000_000 picoseconds.
		Weight::from_parts(38_000_000, 7128)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// The range of component `s` is `[1024, 5120]`.
	fn apply_item_actions_with_signature(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `639`
		//  Estimated: `6084`
		// Minimum execution time: 135_291_000 picoseconds.
		Weight::from_parts(123_608_913, 6084)
			// Standard Error: 360
			.saturating_add(Weight::from_parts(10_409, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// The range of component `s` is `[1, 1024]`.
	fn upsert_page_with_signature(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1618`
		//  Estimated: `7063`
		// Minimum execution time: 112_000_000 picoseconds.
		Weight::from_parts(112_299_676, 7063)
			// Standard Error: 1_943
			.saturating_add(Weight::from_parts(16_486, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	fn delete_page_with_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1616`
		//  Estimated: `7061`
		// Minimum execution time: 112_000_000 picoseconds.
		Weight::from_parts(114_000_000, 7061)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeightChild::get().reads(1_u64))
			.saturating_add(RocksDbWeightChild::get().writes(1_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Handles::MSAIdToDisplayName` (r:1 w:1)
	/// Proof: `Handles::MSAIdToDisplayName` (`max_values`: None, `max_size`: Some(59), added: 2534, mode: `MaxEncodedLen`)
	/// Storage: `Handles::CanonicalBaseHandleToSuffixIndex` (r:1 w:1)
	/// Proof: `Handles::CanonicalBaseHandleToSuffixIndex` (`max_values`: None, `max_size`: Some(53), added: 2528, mode: `MaxEncodedLen`)
	/// Storage: `Handles::CanonicalBaseHandleAndSuffixToMSAId` (r:0 w:1)
	/// Proof: `Handles::CanonicalBaseHandleAndSuffixToMSAId` (`max_values`: None, `max_size`: Some(67), added: 2542, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[3, 30]`.
	fn claim_handle(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `116`
		//  Estimated: `4019`
		// Minimum execution time: 108_678_000 picoseconds.
		Weight::from_parts(110_925_526, 4019)
			// Standard Error: 31_139
			.saturating_add(Weight::from_parts(113_003, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Msa::PublicKeyToMsaId` (r:1 w:0)
	/// Proof: `Msa::PublicKeyToMsaId` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Handles::MSAIdToDisplayName` (r:1 w:1)
	/// Proof: `Handles::MSAIdToDisplayName` (`max_values`: None, `max_size`: Some(59), added: 2534, mode: `MaxEncodedLen`)
	/// Storage: `Handles::CanonicalBaseHandleToSuffixIndex` (r:1 w:1)
	/// Proof: `Handles::CanonicalBaseHandleToSuffixIndex` (`max_values`: None, `max_size`: Some(53), added: 2528, mode: `MaxEncodedLen`)
	/// Storage: `Handles::CanonicalBaseHandleAndSuffixToMSAId` (r:0 w:2)
	/// Proof: `Handles::CanonicalBaseHandleAndSuffixToMSAId` (`max_values`: None, `max_size`: Some(67), added: 2542, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[3, 30]`.
	fn change_handle(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `222 + b * (1 ±0)`
		//  Estimated: `4019`
		// Minimum execution time: 116_507_000 picoseconds.
		Weight::from_parts(118_375_742, 4019)
			// Standard Error: 34_874
			.saturating_add(Weight::from_parts(217_518, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}
