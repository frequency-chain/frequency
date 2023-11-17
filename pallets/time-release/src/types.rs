//! Types for the TimeRelease Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use sp_runtime::{traits::AtLeast32Bit, RuntimeDebug};
use sp_std::cmp::{Eq, PartialEq};

use scale_info::TypeInfo;

/// The release schedule.
///
/// Benefits would be granted gradually, `per_period` amount every `period`
/// of blocks after `start`.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ReleaseSchedule<BlockNumber, Balance: MaxEncodedLen + HasCompact> {
	/// Vesting starting block
	pub start: BlockNumber,
	/// Number of blocks between vest
	pub period: BlockNumber,
	/// Number of vest
	pub period_count: u32,
	/// Amount of tokens to release per vest
	#[codec(compact)]
	pub per_period: Balance,
}

impl<BlockNumber: AtLeast32Bit + Copy, Balance: AtLeast32Bit + MaxEncodedLen + Copy>
	ReleaseSchedule<BlockNumber, Balance>
{
	/// Returns the end of all periods, `None` if calculation overflows.
	pub fn end(&self) -> Option<BlockNumber> {
		// period * period_count + start
		self.period.checked_mul(&self.period_count.into())?.checked_add(&self.start)
	}

	/// Returns all locked amount, `None` if calculation overflows.
	pub fn total_amount(&self) -> Option<Balance> {
		self.per_period.checked_mul(&self.period_count.into())
	}

	/// Returns locked amount for a given `time`.
	///
	/// Note this func assumes schedule is a valid one(non-zero period and
	/// non-overflow total amount), and it should be guaranteed by callers.
	#[allow(clippy::expect_used)]
	pub fn locked_amount(&self, time: BlockNumber) -> Balance {
		// full = (time - start) / period
		// unrealized = period_count - full
		// per_period * unrealized
		let full = time
			.saturating_sub(self.start)
			.checked_div(&self.period)
			.expect("ensured non-zero period; qed");
		let unrealized = self.period_count.saturating_sub(full.unique_saturated_into());
		self.per_period
			.checked_mul(&unrealized.into())
			.expect("ensured non-overflow total amount; qed")
	}
}
