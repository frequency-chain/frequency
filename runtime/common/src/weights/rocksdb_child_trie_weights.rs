//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 48.0.0
//! DATE: 2025-10-22 (Y/M/D)
//! HOSTNAME: `ip-10-173-4-131`, CPU: `Intel(R) Xeon(R) Platinum 8488C`
//!
//! DATABASE: `RocksDb`, RUNTIME: `Frequency`
//! BLOCK-NUM: `BlockId::Number(9443452)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `2`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: ``
//! METRIC: `Average`, WEIGHT-MUL: `1.3`, WEIGHT-ADD: `0`

// Executed Command:
//   ./frequency/target/release/frequency
//   benchmark
//   storage
//   --state-version=1
//   --chain=frequency
//   --base-path=/data
//   --warmups=2
//   --mul=1.3

/// Storage DB weights for the `Frequency` runtime and `RocksDb`.
pub mod constants {
	use frame_support::weights::constants;
	use sp_core::parameter_types;
	use sp_weights::RuntimeDbWeight;

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeightChild: RuntimeDbWeight = RuntimeDbWeight {
			// Time to read one storage item.
			// Calculated by multiplying the *Average* of all values with `1.3` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 61_748, 44_331_136
			//   Average:  2_370_750
			//   Median:   2_255_865
			//   Std-Dev:  1148296.92
			//
			// Percentiles nanoseconds:
			//   99th: 5_622_557
			//   95th: 4_546_929
			//   75th: 3_024_267
			read: 3_081_975 * constants::WEIGHT_REF_TIME_PER_NANOS,

			// Time to write one storage item.
			// Calculated by multiplying the *Average* of all values with `1.3` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 6_606, 21_531_239
			//   Average:  31_890
			//   Median:   29_097
			//   Std-Dev:  93545.35
			//
			// Percentiles nanoseconds:
			//   99th: 64_921
			//   95th: 50_484
			//   75th: 35_177
			write: 41_457 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use sp_weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}