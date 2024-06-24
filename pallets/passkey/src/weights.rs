
//! Autogenerated weights for `pallet_passkey`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-06-24, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `AramikMarkarianNewMacBookPro.local`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("frequency-bench")`, DB CACHE: `1024`

// Executed Command:
// ./scripts/../target/bench-dev/frequency
// benchmark
// pallet
// --pallet=pallet_passkey
// --extrinsic
// *
// --chain=frequency-bench
// --heap-pages=4096
// --wasm-execution=compiled
// --additional-trie-layers=5
// --steps=20
// --repeat=10
// --output=./scripts/../pallets/passkey/src/weights.rs
// --template=./scripts/../.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_passkey`.
pub trait WeightInfo {
	fn validate() -> Weight;
	fn pre_dispatch() -> Weight;
}

/// Weights for `pallet_passkey` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn validate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `39`
		//  Estimated: `5078`
		// Minimum execution time: 42_000_000 picoseconds.
		Weight::from_parts(43_000_000, 5078)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn pre_dispatch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `5078`
		// Minimum execution time: 45_000_000 picoseconds.
		Weight::from_parts(46_000_000, 5078)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn validate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `39`
		//  Estimated: `5078`
		// Minimum execution time: 42_000_000 picoseconds.
		Weight::from_parts(43_000_000, 5078)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn pre_dispatch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `5078`
		// Minimum execution time: 45_000_000 picoseconds.
		Weight::from_parts(46_000_000, 5078)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
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
	fn test_validate() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5078
		);
	}
	#[test]
	fn test_pre_dispatch() {
		assert!(
			BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5078
		);
	}
}