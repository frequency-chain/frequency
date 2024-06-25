use crate::constants::currency::CENTS;

use frame_support::{
	sp_runtime::Perbill,
	weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};

use common_primitives::node::Balance;

use super::weights::extrinsic_weights::ExtrinsicBaseWeight;

use smallvec::smallvec;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]` 1_000_000_000_000
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.

/// Warning: Changing this function will also change the static capacity weights.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// In Polkadot extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT
		let p = CENTS;
		let q = 10 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

#[cfg(test)]
mod test {
	use super::WeightToFee;
	use crate::{
		constants::{
			currency::{CENTS, DOLLARS, MILLICENTS},
			MAXIMUM_BLOCK_WEIGHT,
		},
		fee::Balance,
		weights::extrinsic_weights::ExtrinsicBaseWeight,
	};

	use frame_support::weights::WeightToFee as WeightToFeeT;

	#[test]
	// Test that the fee for `MAXIMUM_BLOCK_WEIGHT` of weigh has sane bounds.
	fn full_block_fee_is_correct() {
		let full_block = WeightToFee::weight_to_fee(&MAXIMUM_BLOCK_WEIGHT);
		// A bounded assertion to consider changes in generated extrinsic base weight.
		assert!(full_block >= 2 * 150 * CENTS);
		assert!(full_block <= 10 * DOLLARS);
	}
	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/10 of a CENT
		let x = WeightToFee::weight_to_fee(&ExtrinsicBaseWeight::get());
		let y = CENTS / 10;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}

	#[test]
	fn check_weight() {
		let p = CENTS / 10;
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());

		assert_eq!(p, 100_000);

		assert!(q >= 65_000_000);
		assert!(q <= 100_000_000);

		assert_eq!(p / q, Balance::from(0u128));
	}
}
