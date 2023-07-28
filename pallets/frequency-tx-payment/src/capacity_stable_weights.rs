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
	// Storage: Msa PayloadSignatureBucketCount (r:1 w:1)
	// Storage: Msa PayloadSignatureRegistry (r:1 w:1)
	// Storage: Msa PublicKeyToMsaId (r:2 w:1)
	// Storage: Msa ProviderToRegistryEntry (r:1 w:0)
	// Storage: Msa CurrentMsaIdentifierMaximum (r:1 w:1)
	// Storage: Msa PublicKeyCountForMsaId (r:1 w:1)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:1)
	// Storage: Schemas CurrentSchemaIdentifierMaximum (r:1 w:0)
	fn create_sponsored_account_with_delegation(s: u32) -> Weight {
		Weight::from_parts(100_556_500 as u64, 0)
			// Standard Error: 19_778
			.saturating_add(Weight::from_parts(120_447 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	// Storage: Msa PayloadSignatureBucketCount (r:1 w:1)
	// Storage: Msa PayloadSignatureRegistry (r:2 w:2)
	// Storage: Msa PublicKeyToMsaId (r:2 w:1)
	// Storage: Msa PublicKeyCountForMsaId (r:1 w:1)
	fn add_public_key_to_msa() -> Weight {
		Weight::from_parts(147_786_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Msa PayloadSignatureBucketCount (r:1 w:1)
	// Storage: Msa PayloadSignatureRegistry (r:1 w:1)
	// Storage: Msa PublicKeyToMsaId (r:2 w:0)
	// Storage: Msa ProviderToRegistryEntry (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:1)
	// Storage: Schemas CurrentSchemaIdentifierMaximum (r:1 w:0)
	fn grant_delegation(s: u32) -> Weight {
		Weight::from_parts(94_743_045 as u64, 0)
			// Standard Error: 19_748
			.saturating_add(Weight::from_parts(125_241 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
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
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	// Storage: Messages Messages (r:1 w:1)
	fn add_onchain_message(n: u32) -> Weight {
		Weight::from_parts(139_432_286 as u64, 0)
			// Standard Error: 43
			.saturating_add(Weight::from_parts(1_441 as u64, 0).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Messages Messages (r:1 w:1)
	fn add_ipfs_message() -> Weight {
		Weight::from_parts(131_669_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	// Storage: unknown [0xbd1557c8db6bd8599a811a7175fbc2fc6400] (r:1 w:1)
	fn apply_item_actions(s: u32) -> Weight {
		Weight::from_parts(66_026_301 as u64, 0)
			// Standard Error: 161
			.saturating_add(Weight::from_parts(2_145 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	// Storage: unknown [0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1] (r:1 w:1)
	fn upsert_page(s: u32) -> Weight {
		Weight::from_parts(23_029_186 as u64, 0)
			// Standard Error: 53
			.saturating_add(Weight::from_parts(339 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Msa DelegatorAndProviderToDelegation (r:1 w:0)
	// Storage: unknown [0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1] (r:1 w:1)
	fn delete_page() -> Weight {
		Weight::from_parts(26_000_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: unknown [0xbd1557c8db6bd8599a811a7175fbc2fc6400] (r:1 w:1)
	fn apply_item_actions_with_signature(s: u32) -> Weight {
		Weight::from_parts(105_921_191 as u64, 0)
			// Standard Error: 267
			.saturating_add(Weight::from_parts(6_150 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: unknown [0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1] (r:1 w:1)
	fn upsert_page_with_signature(s: u32) -> Weight {
		Weight::from_parts(61_324_707 as u64, 0)
			// Standard Error: 249
			.saturating_add(Weight::from_parts(4_406 as u64, 0).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Schemas Schemas (r:1 w:0)
	// Storage: unknown [0x0763c98381dc89abe38627fe2f98cb7af1577fbf1d628fdddb4ebfc6e8d95fb1] (r:1 w:1)
	fn delete_page_with_signature() -> Weight {
		Weight::from_parts(65_000_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Handles MSAIdToDisplayName (r:1 w:1)
	// Storage: Handles CanonicalBaseHandleToSuffixIndex (r:1 w:1)
	// Storage: Handles CanonicalBaseHandleAndSuffixToMSAId (r:0 w:1)
	fn claim_handle(b: u32) -> Weight {
		Weight::from_parts(90_537_753 as u64, 0)
			// Standard Error: 27_078
			.saturating_add(Weight::from_parts(104_522 as u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Msa PublicKeyToMsaId (r:1 w:0)
	// Storage: Handles MSAIdToDisplayName (r:1 w:1)
	// Storage: Handles CanonicalBaseHandleToSuffixIndex (r:1 w:1)
	// Storage: Handles CanonicalBaseHandleAndSuffixToMSAId (r:0 w:1)
	fn change_handle(b: u32) -> Weight {
		Weight::from_parts(90_537_753 as u64, 0)
			// Standard Error: 27_078
			.saturating_add(Weight::from_parts(104_522 as u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}
