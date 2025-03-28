//! Types for the TimeRelease Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use sp_runtime::{traits::AtLeast32Bit, DispatchError, RuntimeDebug};
use sp_std::{
	boxed::Box,
	cmp::{Eq, PartialEq},
};

use scale_info::TypeInfo;

/// Alias for a schedule identifier
pub type ScheduleName = [u8; 32];

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

	/// Returns all frozen amount, `None` if calculation overflows.
	pub fn total_amount(&self) -> Option<Balance> {
		self.per_period.checked_mul(&self.period_count.into())
	}

	/// Returns frozen amount for a given `time`.
	///
	/// Note this func assumes schedule is a valid one(non-zero period and
	/// non-overflow total amount), and it should be guaranteed by callers.
	#[allow(clippy::expect_used)]
	pub fn frozen_amount(&self, time: BlockNumber) -> Balance {
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

/// A trait that defines a scheduler provider for scheduling calls to be executed at a specific block number.
pub trait SchedulerProviderTrait<Origin, BlockNumber, Call> {
	/// Schedules a call to be executed at a specified block number.
	///
	/// # Returns
	/// - `Ok(())` if the call was successfully scheduled.
	/// - `Err(DispatchError)` if there was an error scheduling the call.
	///
	/// # Errors
	/// This function may return a `DispatchError` for various reasons, such as:
	/// - Insufficient permissions or invalid origin.
	/// - Invalid block number or scheduling conflicts.
	/// - Other runtime-specific errors.
	fn schedule(
		origin: Origin,
		id: ScheduleName,
		when: BlockNumber,
		call: Box<Call>,
	) -> Result<(), DispatchError>;

	/// Cancels a scheduled call with an specific schedule-name.
	///
	/// # Returns
	/// - `Ok(())` if the call was successfully canceled.
	/// - `Err(DispatchError)` if there was an error canceling the call.
	fn cancel(origin: Origin, id: ScheduleName) -> Result<(), DispatchError>;
}
