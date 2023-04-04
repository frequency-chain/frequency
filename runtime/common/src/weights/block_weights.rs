
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-04 (Y/M/D)
//! HOSTNAME: `benchmark-runner-p5rt6-4xlb6`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//!
//! SHORT-NAME: `block`, LONG-NAME: `BlockExecution`, RUNTIME: `Frequency Local Testnet`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `runtime/common/src/weights`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   ./scripts/../target/production/frequency
//   benchmark
//   overhead
//   --execution=wasm
//   --wasm-execution=compiled
//   --weight-path=runtime/common/src/weights
//   --chain=dev
//   --warmup=10
//   --repeat=100

use sp_core::parameter_types;
use sp_weights::{constants::WEIGHT_REF_TIME_PER_NANOS, Weight};

parameter_types! {
	/// Time to execute an empty block.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 334_303, 379_043
	///   Average:  345_307
	///   Median:   340_464
	///   Std-Dev:  10642.42
	///
	/// Percentiles nanoseconds:
	///   99th: 371_520
	///   95th: 366_717
	///   75th: 352_006
	pub const BlockExecutionWeight: Weight =
		Weight::from_ref_time(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(345_307));
}

#[cfg(test)]
mod test_weights {
	use sp_weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::BlockExecutionWeight::get();

		// At least 100 µs.
		assert!(
			w.ref_time() >= 100u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
			"Weight should be at least 100 µs."
		);
		// At most 50 ms.
		assert!(
			w.ref_time() <= 50u64 * constants::WEIGHT_REF_TIME_PER_MILLIS,
			"Weight should be at most 50 ms."
		);
	}
}
