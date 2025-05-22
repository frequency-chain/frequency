use frame_support::parameter_types;
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::FixedWeightBounds;

use crate::RuntimeCall;

parameter_types! {
	/// The cost of executing a single XCM instruction
	pub const UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
}

/// The XCM weigher used to estimate total weight for execution
pub type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
