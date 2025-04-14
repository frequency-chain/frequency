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
use sp_std::marker::PhantomData;

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
	/// Storage: Msa PayloadSignatureRegistryList (r:2 w:2)
	/// Proof: Msa PayloadSignatureRegistryList (max_values: Some(50000), max_size: Some(144), added: 2124, mode: MaxEncodedLen)
	/// Storage: Msa PayloadSignatureRegistryPointer (r:1 w:1)
	/// Proof: Msa PayloadSignatureRegistryPointer (max_values: Some(1), max_size: Some(140), added: 635, mode: MaxEncodedLen)
	/// Storage: Msa PublicKeyToMsaId (r:2 w:1)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa ProviderToRegistryEntry (r:1 w:0)
	/// Proof: Msa ProviderToRegistryEntry (max_values: None, max_size: Some(33), added: 2508, mode: MaxEncodedLen)
	/// Storage: Msa CurrentMsaIdentifierMaximum (r:1 w:1)
	/// Proof: Msa CurrentMsaIdentifierMaximum (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Msa PublicKeyCountForMsaId (r:1 w:1)
	/// Proof: Msa PublicKeyCountForMsaId (max_values: None, max_size: Some(17), added: 2492, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:1)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: Schemas CurrentSchemaIdentifierMaximum (r:1 w:0)
	/// Proof Skipped: Schemas CurrentSchemaIdentifierMaximum (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `s` is `[0, 30]`.
	fn create_sponsored_account_with_delegation(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1393`
		//  Estimated: `14946`
		// Minimum execution time: 119_384_000 picoseconds.
		Weight::from_parts(123_084_817, 14946)
			// Standard Error: 20_681
			.saturating_add(Weight::from_parts(138_668, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(10_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: Msa PayloadSignatureRegistryList (r:4 w:4)
	/// Proof: Msa PayloadSignatureRegistryList (max_values: Some(50000), max_size: Some(144), added: 2124, mode: MaxEncodedLen)
	/// Storage: Msa PayloadSignatureRegistryPointer (r:1 w:1)
	/// Proof: Msa PayloadSignatureRegistryPointer (max_values: Some(1), max_size: Some(140), added: 635, mode: MaxEncodedLen)
	/// Storage: Msa PublicKeyToMsaId (r:2 w:1)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa PublicKeyCountForMsaId (r:1 w:1)
	/// Proof: Msa PublicKeyCountForMsaId (max_values: None, max_size: Some(17), added: 2492, mode: MaxEncodedLen)
	fn add_public_key_to_msa() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1677`
		//  Estimated: `18396`
		// Minimum execution time: 175_288_000 picoseconds.
		Weight::from_parts(177_629_000, 18396)
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: Msa PayloadSignatureRegistryList (r:2 w:2)
	/// Proof: Msa PayloadSignatureRegistryList (max_values: Some(50000), max_size: Some(144), added: 2124, mode: MaxEncodedLen)
	/// Storage: Msa PayloadSignatureRegistryPointer (r:1 w:1)
	/// Proof: Msa PayloadSignatureRegistryPointer (max_values: Some(1), max_size: Some(140), added: 635, mode: MaxEncodedLen)
	/// Storage: Msa PublicKeyToMsaId (r:2 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa ProviderToRegistryEntry (r:1 w:0)
	/// Proof: Msa ProviderToRegistryEntry (max_values: None, max_size: Some(33), added: 2508, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:1)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: Schemas CurrentSchemaIdentifierMaximum (r:1 w:0)
	/// Proof Skipped: Schemas CurrentSchemaIdentifierMaximum (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `s` is `[0, 30]`.
	fn grant_delegation(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1443`
		//  Estimated: `14946`
		// Minimum execution time: 111_571_000 picoseconds.
		Weight::from_parts(115_947_313, 14946)
			// Standard Error: 18_186
			.saturating_add(Weight::from_parts(192_710, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: Messages Messages (r:1 w:1)
	/// Proof Skipped: Messages Messages (max_values: None, max_size: None, mode: Measured)
	/// The range of component `n` is `[0, 51199]`.
	fn add_onchain_message(n: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `46773`
		//  Estimated: `59148`
		// Minimum execution time: 164_198_000 picoseconds.
		Weight::from_parts(174_064_230, 59148)
			// Standard Error: 48
			.saturating_add(Weight::from_parts(1_527, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Messages Messages (r:1 w:1)
	/// Proof Skipped: Messages Messages (max_values: None, max_size: None, mode: Measured)
	fn add_ipfs_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `36289`
		//  Estimated: `48664`
		// Minimum execution time: 156_648_000 picoseconds.
		Weight::from_parts(159_242_000, 48664)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: unknown `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// Proof Skipped: unknown `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// The range of component `s` is `[1, 5121]`.
	fn apply_item_actions(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `33370`
		//  Estimated: `45745`
		// Minimum execution time: 105_792_000 picoseconds.
		Weight::from_parts(104_137_090, 45745)
			// Standard Error: 315
			.saturating_add(Weight::from_parts(7_325, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof Skipped: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// The range of component `s` is `[1, 1024]`.
	fn upsert_page(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `416`
		//  Estimated: `12791`
		// Minimum execution time: 30_996_000 picoseconds.
		Weight::from_parts(32_297_181, 12791)
			// Standard Error: 203
			.saturating_add(Weight::from_parts(594, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	/// Proof: Msa DelegatorAndProviderToDelegation (max_values: None, max_size: Some(217), added: 2692, mode: MaxEncodedLen)
	/// Storage: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof Skipped: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	fn delete_page() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1575`
		//  Estimated: `13950`
		// Minimum execution time: 36_810_000 picoseconds.
		Weight::from_parts(37_792_000, 13950)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: unknown `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// Proof Skipped: unknown `0xbd1557c8db6bd8599a811a7175fbc2fc6400` (r:1 w:1)
	/// The range of component `s` is `[1, 5121]`.
	fn apply_item_actions_with_signature(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `33377`
		//  Estimated: `45752`
		// Minimum execution time: 165_956_000 picoseconds.
		Weight::from_parts(158_947_612, 45752)
			// Standard Error: 426
			.saturating_add(Weight::from_parts(13_664, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof Skipped: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// The range of component `s` is `[1, 1024]`.
	fn upsert_page_with_signature(s: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `349`
		//  Estimated: `12724`
		// Minimum execution time: 85_227_000 picoseconds.
		Weight::from_parts(87_768_259, 12724)
			// Standard Error: 543
			.saturating_add(Weight::from_parts(6_648, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Schemas Schemas (r:1 w:0)
	/// Proof Skipped: Schemas Schemas (max_values: None, max_size: None, mode: Measured)
	/// Storage: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	/// Proof Skipped: unknown `0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1` (r:1 w:1)
	fn delete_page_with_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1508`
		//  Estimated: `13883`
		// Minimum execution time: 88_824_000 picoseconds.
		Weight::from_parts(90_631_000, 13883)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Handles MSAIdToDisplayName (r:1 w:1)
	/// Proof: Handles MSAIdToDisplayName (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Handles CanonicalBaseHandleToSuffixIndex (r:1 w:1)
	/// Proof: Handles CanonicalBaseHandleToSuffixIndex (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
	/// Storage: Handles CanonicalBaseHandleAndSuffixToMSAId (r:0 w:1)
	/// Proof: Handles CanonicalBaseHandleAndSuffixToMSAId (max_values: None, max_size: Some(67), added: 2542, mode: MaxEncodedLen)
	/// The range of component `b` is `[3, 30]`.
	fn claim_handle(b: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `191`
		//  Estimated: `12434`
		// Minimum execution time: 83_175_000 picoseconds.
		Weight::from_parts(85_480_476, 12434)
			// Standard Error: 25_131
			.saturating_add(Weight::from_parts(107_272, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Proof: Msa PublicKeyToMsaId (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Handles MSAIdToDisplayName (r:1 w:1)
	/// Proof: Handles MSAIdToDisplayName (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Handles CanonicalBaseHandleToSuffixIndex (r:1 w:1)
	/// Proof: Handles CanonicalBaseHandleToSuffixIndex (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
	/// Storage: Handles CanonicalBaseHandleAndSuffixToMSAId (r:0 w:2)
	/// Proof: Handles CanonicalBaseHandleAndSuffixToMSAId (max_values: None, max_size: Some(67), added: 2542, mode: MaxEncodedLen)
	/// The range of component `b` is `[3, 30]`.
	fn change_handle(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `297 + b * (1 ±0)`
		//  Estimated: `12434`
		// Minimum execution time: 93_749_000 picoseconds.
		Weight::from_parts(95_748_064, 12434)
			// Standard Error: 9_821
			.saturating_add(Weight::from_parts(212_118, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}
