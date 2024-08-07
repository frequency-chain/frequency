
//! Autogenerated weights for `pallet_schemas`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-08-01, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-10-173-4-91`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("frequency-bench")`, DB CACHE: `1024`

// Executed Command:
// ./scripts/../target/release/frequency
// benchmark
// pallet
// --pallet=pallet_schemas
// --extrinsic
// *
// --chain=frequency-bench
// --heap-pages=4096
// --wasm-execution=compiled
// --additional-trie-layers=5
// --steps=20
// --repeat=10
// --output=./scripts/../pallets/schemas/src/weights.rs
// --template=./scripts/../.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_schemas`.
pub trait WeightInfo {
	fn create_schema(m: u32, ) -> Weight;
	fn create_schema_via_governance(m: u32, ) -> Weight;
	fn propose_to_create_schema(m: u32, ) -> Weight;
	fn create_schema_v2(m: u32, ) -> Weight;
	fn create_schema_v3(m: u32, ) -> Weight;
	fn set_max_schema_model_bytes() -> Weight;
	fn create_schema_via_governance_v2(m: u32, ) -> Weight;
	fn propose_to_create_schema_v2(m: u32, ) -> Weight;
	fn propose_to_create_schema_name() -> Weight;
	fn create_schema_name_via_governance() -> Weight;
}

/// Weights for `pallet_schemas` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_340_000 picoseconds.
		Weight::from_parts(12_484_000, 2974)
			// Standard Error: 49
			.saturating_add(Weight::from_parts(35_365, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_via_governance(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_285_000 picoseconds.
		Weight::from_parts(12_482_000, 2974)
			// Standard Error: 40
			.saturating_add(Weight::from_parts(36_168, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[16, 65499]`.
	fn propose_to_create_schema(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `230`
		//  Estimated: `5180`
		// Minimum execution time: 16_805_000 picoseconds.
		Weight::from_parts(4_805_927, 5180)
			// Standard Error: 32
			.saturating_add(Weight::from_parts(3_107, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_415_000 picoseconds.
		Weight::from_parts(12_627_000, 2974)
			// Standard Error: 41
			.saturating_add(Weight::from_parts(35_351, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_v3(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `5552`
		// Minimum execution time: 20_338_000 picoseconds.
		Weight::from_parts(3_529_500, 5552)
			// Standard Error: 64
			.saturating_add(Weight::from_parts(35_780, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:0 w:1)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn set_max_schema_model_bytes() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_211_000 picoseconds.
		Weight::from_parts(4_554_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_via_governance_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `5552`
		// Minimum execution time: 19_955_000 picoseconds.
		Weight::from_parts(20_175_000, 5552)
			// Standard Error: 69
			.saturating_add(Weight::from_parts(36_583, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[16, 65499]`.
	fn propose_to_create_schema_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `230`
		//  Estimated: `5180`
		// Minimum execution time: 17_265_000 picoseconds.
		Weight::from_parts(5_631_110, 5180)
			// Standard Error: 33
			.saturating_add(Weight::from_parts(3_098, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn propose_to_create_schema_name() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `433`
		//  Estimated: `5383`
		// Minimum execution time: 22_947_000 picoseconds.
		Weight::from_parts(23_512_000, 5383)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	fn create_schema_name_via_governance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `203`
		//  Estimated: `5552`
		// Minimum execution time: 14_864_000 picoseconds.
		Weight::from_parts(15_656_000, 5552)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_340_000 picoseconds.
		Weight::from_parts(12_484_000, 2974)
			// Standard Error: 49
			.saturating_add(Weight::from_parts(35_365, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_via_governance(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_285_000 picoseconds.
		Weight::from_parts(12_482_000, 2974)
			// Standard Error: 40
			.saturating_add(Weight::from_parts(36_168, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[16, 65499]`.
	fn propose_to_create_schema(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `230`
		//  Estimated: `5180`
		// Minimum execution time: 16_805_000 picoseconds.
		Weight::from_parts(4_805_927, 5180)
			// Standard Error: 32
			.saturating_add(Weight::from_parts(3_107, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `2974`
		// Minimum execution time: 12_415_000 picoseconds.
		Weight::from_parts(12_627_000, 2974)
			// Standard Error: 41
			.saturating_add(Weight::from_parts(35_351, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_v3(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `5552`
		// Minimum execution time: 20_338_000 picoseconds.
		Weight::from_parts(3_529_500, 5552)
			// Standard Error: 64
			.saturating_add(Weight::from_parts(35_780, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:0 w:1)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn set_max_schema_model_bytes() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_211_000 picoseconds.
		Weight::from_parts(4_554_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Schemas::GovernanceSchemaModelMaxBytes` (r:1 w:0)
	/// Proof: `Schemas::GovernanceSchemaModelMaxBytes` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::CurrentSchemaIdentifierMaximum` (r:1 w:1)
	/// Proof: `Schemas::CurrentSchemaIdentifierMaximum` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaInfos` (r:0 w:1)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaPayloads` (r:0 w:1)
	/// Proof: `Schemas::SchemaPayloads` (`max_values`: None, `max_size`: Some(65514), added: 67989, mode: `MaxEncodedLen`)
	/// The range of component `m` is `[16, 65499]`.
	fn create_schema_via_governance_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `5552`
		// Minimum execution time: 19_955_000 picoseconds.
		Weight::from_parts(20_175_000, 5552)
			// Standard Error: 69
			.saturating_add(Weight::from_parts(36_583, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[16, 65499]`.
	fn propose_to_create_schema_v2(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `230`
		//  Estimated: `5180`
		// Minimum execution time: 17_265_000 picoseconds.
		Weight::from_parts(5_631_110, 5180)
			// Standard Error: 33
			.saturating_add(Weight::from_parts(3_098, 0).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn propose_to_create_schema_name() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `433`
		//  Estimated: `5383`
		// Minimum execution time: 22_947_000 picoseconds.
		Weight::from_parts(23_512_000, 5383)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Schemas::SchemaInfos` (r:1 w:0)
	/// Proof: `Schemas::SchemaInfos` (`max_values`: None, `max_size`: Some(15), added: 2490, mode: `MaxEncodedLen`)
	/// Storage: `Schemas::SchemaNameToIds` (r:1 w:1)
	/// Proof: `Schemas::SchemaNameToIds` (`max_values`: None, `max_size`: Some(602), added: 3077, mode: `MaxEncodedLen`)
	fn create_schema_name_via_governance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `203`
		//  Estimated: `5552`
		// Minimum execution time: 14_864_000 picoseconds.
		Weight::from_parts(15_656_000, 5552)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}


#[cfg(test)]
mod tests {
  use frame_support::{traits::Get, weights::Weight, dispatch::DispatchClass};
  use common_runtime::constants::{MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO};
  use common_runtime::weights::extrinsic_weights::ExtrinsicBaseWeight;

  struct BlockWeights;
  impl Get<frame_system::limits::BlockWeights> for BlockWeights {
  	fn get() -> frame_system::limits::BlockWeights {
  		frame_system::limits::BlockWeights::builder()
  			.base_block(Weight::zero())
  			.for_class(DispatchClass::all(), |weights| {
  				weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
  			})
  			.for_class(DispatchClass::non_mandatory(), |weights| {
  				weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
  			})
  			.build_or_panic()
  	}
  }

	#[test]
	fn test_create_schema() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2974
		);
	}
	#[test]
	fn test_create_schema_via_governance() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2974
		);
	}
	#[test]
	fn test_propose_to_create_schema() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5180
		);
	}
	#[test]
	fn test_create_schema_v2() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2974
		);
	}
	#[test]
	fn test_create_schema_v3() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5552
		);
	}
	#[test]
	fn test_create_schema_via_governance_v2() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5552
		);
	}
	#[test]
	fn test_propose_to_create_schema_v2() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5180
		);
	}
	#[test]
	fn test_propose_to_create_schema_name() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5383
		);
	}
	#[test]
	fn test_create_schema_name_via_governance() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5552
		);
	}
}
