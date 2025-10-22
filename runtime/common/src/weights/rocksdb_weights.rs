//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-08-22 (Y/M/D)
//! HOSTNAME: `ip-10-173-4-131`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//!
//! DATABASE: `RocksDb`, RUNTIME: `Frequency`
//! BLOCK-NUM: `BlockId::Number(4413540)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `2`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: ``
//! METRIC: `Average`, WEIGHT-MUL: `1.3`, WEIGHT-ADD: `0`

// Executed Command:
//   ./target/release/frequency
//   benchmark
//   storage
//   --state-version=1
//   --chain=frequency
//   --base-path=/data
//   --include-child-trees
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
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			// Time to read one storage item.
			// Calculated by multiplying the *Average* of all values with `1.3` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 1_676, 3_967_371
			//   Average:  68_610
			//   Median:   55_534
			//   Std-Dev:  104861.07
			//
			// Percentiles nanoseconds:
			//   99th: 743_669
			//   95th: 82_813
			//   75th: 62_747
			read: 89_193 * constants::WEIGHT_REF_TIME_PER_NANOS,

			// Time to write one storage item.
			// Calculated by multiplying the *Average* of all values with `1.3` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 6_945, 27_330_702
			//   Average:  94_298
			//   Median:   104_840
			//   Std-Dev:  89548.06
			//
			//   99th: 167_407
			//   95th: 146_669
			//   75th: 121_699
			write: 122_587 * constants::WEIGHT_REF_TIME_PER_NANOS,
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
