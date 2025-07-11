use crate::{xcm::parameters::RootLocation, TreasuryAccount};
use staging_xcm_builder::{SendXcmFeeToAccount, XcmFeeManagerFromComponents};

use super::AssetTransactors;
use frame_support::traits::Equals;
/// Locations that will not be charged fees in the executor,
/// either execution or delivery.
/// We only waive fees for system functions, which these locations represent.
pub type WaivedLocations = (Equals<RootLocation>,);

pub type FeeManager = XcmFeeManagerFromComponents<
	WaivedLocations,
	SendXcmFeeToAccount<AssetTransactors, TreasuryAccount>,
>;
