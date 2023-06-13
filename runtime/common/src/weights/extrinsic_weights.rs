
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-13 (Y/M/D)
//! HOSTNAME: `Matthew-Orris-M2-Pro.local`, CPU: `<UNKNOWN>`
//!
//! SHORT-NAME: `extrinsic`, LONG-NAME: `ExtrinsicBase`, RUNTIME: `Frequency Development (No Relay)`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `runtime/common/src/weights`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   ./scripts/../target/release/frequency
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
	/// Time to execute a NO-OP extrinsic, for example `System::remark`.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 70_240, 75_744
	///   Average:  71_562
	///   Median:   71_043
	///   Std-Dev:  1373.86
	///
	/// Percentiles nanoseconds:
	///   99th: 75_228
	///   95th: 74_133
	///   75th: 72_270
	pub const ExtrinsicBaseWeight: Weight =
		Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(71_562), 0);
}

#[cfg(test)]
mod test_weights {
	use sp_weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::ExtrinsicBaseWeight::get();

		// At least 10 µs.
		assert!(
			w.ref_time() >= 10u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
			"Weight should be at least 10 µs."
		);
		// At most 1 ms.
		assert!(
			w.ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
			"Weight should be at most 1 ms."
		);
	}
}
