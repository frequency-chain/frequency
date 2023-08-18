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

use frame_support::{traits::Get, weights::{Weight, constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_NANOS}}};
use sp_std::marker::PhantomData;

/// The base fee for extrinsics is calculated by running benchmarks.
/// Capacity needs the base fee to remain stable and not change when benchmarks are run.
/// CAPACITY_EXTRINSIC_BASE_WEIGHT is a snapshot of the ExtrinsicBaseWeight
/// taken from: runtime/common/src/weights/extrinsic_weights.rs
///   THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
///   DATE: 2023-04-05 (Y/M/D)
///   HOSTNAME: `benchmark-runner-p5rt6-vk9q8`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
///
///   SHORT-NAME: `extrinsic`, LONG-NAME: `ExtrinsicBase`, RUNTIME: `Frequency Local Testnet`
///   WARMUPS: `10`, REPEAT: `100`
///   WEIGHT-PATH: `runtime/common/src/weights`
///   WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`
///
///   Executed Command:
///     ./scripts/../target/release/frequency
///     benchmark
///     overhead
///     --execution=wasm
///     --wasm-execution=compiled
///     --weight-path=runtime/common/src/weights
///     --chain=dev
///     --warmup=10
///     --repeat=100,
///
/// Time to execute a NO-OP extrinsic, for example `System::remark`.
/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
///
/// Stats nanoseconds:
/// - Min, Max: 90_148, 102_526
/// - Average:  90_764
/// - Median:   90_507
/// - Std-Dev:  1449.85
///
/// Percentiles nanoseconds:
/// - 99th: 96_896
/// - 95th: 91_299
/// - 75th: 90_626
pub const CAPACITY_EXTRINSIC_BASE_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(90_764), 0);

/// Weight functions needed for pallet_msa.
pub trait WeightInfo {
	// MSA
	fn create_sponsored_account_with_delegation(s: u32) -> Weight;
	fn add_public_key_to_msa() -> Weight;
	fn grant_delegation(s: u32) -> Weight;
	fn grant_schema_permissions(s: u32) -> Weight;
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
		// Minimum execution time: 119_554_000 picoseconds.
		Weight::from_parts(124_216_637, 14946)
			// Standard Error: 26_556
			.saturating_add(Weight::from_parts(129_688, 0).saturating_mul(s.into()))
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
		//  Measured:  `1654`
		//  Estimated: `18396`
		// Minimum execution time: 184_446_000 picoseconds.
		Weight::from_parts(188_662_000, 18396)
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
		//  Measured:  `1447`
		//  Estimated: `14946`
		// Minimum execution time: 115_100_000 picoseconds.
		Weight::from_parts(120_342_076, 14946)
			// Standard Error: 35_029
			.saturating_add(Weight::from_parts(34_990, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:1)
	// Storage: Schemas CurrentSchemaIdentifierMaximum (r:1 w:0)
	fn grant_schema_permissions(s: u32) -> Weight {
		Weight::from_parts(26_682_873 as u64, 0)
			// Standard Error: 7_236
			.saturating_add(Weight::from_parts(63_887 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
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
		// Minimum execution time: 180_329_000 picoseconds.
		Weight::from_parts(179_112_822, 59148)
			// Standard Error: 52
			.saturating_add(Weight::from_parts(1_852, 0).saturating_mul(n.into()))
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
		// Minimum execution time: 169_213_000 picoseconds.
		Weight::from_parts(174_485_000, 48664)
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
		// Minimum execution time: 107_769_000 picoseconds.
		Weight::from_parts(105_222_871, 45745)
			// Standard Error: 361
			.saturating_add(Weight::from_parts(8_115, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 31_259_000 picoseconds.
		Weight::from_parts(32_661_101, 12791)
			// Standard Error: 203
			.saturating_add(Weight::from_parts(786, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 37_460_000 picoseconds.
		Weight::from_parts(39_471_000, 13950)
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
		// Minimum execution time: 175_937_000 picoseconds.
		Weight::from_parts(169_857_770, 45752)
			// Standard Error: 561
			.saturating_add(Weight::from_parts(15_494, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 87_687_000 picoseconds.
		Weight::from_parts(91_158_457, 12724)
			// Standard Error: 668
			.saturating_add(Weight::from_parts(7_009, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 89_775_000 picoseconds.
		Weight::from_parts(92_238_000, 13883)
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
		// Minimum execution time: 83_385_000 picoseconds.
		Weight::from_parts(85_563_974, 12434)
			// Standard Error: 14_428
			.saturating_add(Weight::from_parts(105_821, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	/// Storage: Handles MSAIdToDisplayName (r:1 w:1)
	/// Storage: Handles CanonicalBaseHandleToSuffixIndex (r:1 w:1)
	/// Storage: Handles CanonicalBaseHandleAndSuffixToMSAId (r:0 w:2)
	/// The range of component `b` is `[3, 30]`.
	fn change_handle(b: u32, ) -> Weight {
		Weight::from_parts(96_221_224, 12434)
			// Standard Error: 9_682
			.saturating_add(Weight::from_parts(193_495, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}
