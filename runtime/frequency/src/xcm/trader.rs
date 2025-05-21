use crate::{
	AccountId, Balances, Runtime,
};
use frame_support::parameter_types;
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{FixedRateOfFungible, UsingComponents};
use polkadot_runtime_common::impls::ToAuthor;

use crate::xcm::constants::{RelayPerSecondAndByte, HereLocation};
use common_runtime::fee::WeightToFee;

/// The full Trader used for pricing XCM execution and delivery
pub type Trader = (
	FixedRateOfFungible<RelayPerSecondAndByte, ()>,
	UsingComponents<WeightToFee, HereLocation, AccountId, Balances, ToAuthor<Runtime>>,
);