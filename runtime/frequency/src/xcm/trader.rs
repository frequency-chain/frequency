use crate::{AccountId, Balances, Runtime};
use polkadot_runtime_common::impls::ToAuthor;
use staging_xcm_builder::{FixedRateOfFungible, UsingComponents};

use crate::xcm::parameters::{HereLocation, RelayPerSecondAndByte};
use common_runtime::fee::WeightToFee;

/// The full Trader used for pricing XCM execution and delivery
pub type Trader = (
	FixedRateOfFungible<RelayPerSecondAndByte, ()>,
	UsingComponents<WeightToFee, HereLocation, AccountId, Balances, ToAuthor<Runtime>>,
);
