use crate::PolkadotXcm;
use frame_support::traits::{ConstU32, Contains, Everything};

use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{
	AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, DenyRecursively, DenyReserveTransferToRelayChain, DenyThenTry,
	TakeWeightCredit, TrailingSetTopicAsId, WithComputedOrigin,
};

use parachains_common::xcm_config::ParentRelayOrSiblingParachains;

use crate::xcm::parameters::UniversalLocation;

pub struct ParentOrParentsExecutivePlurality;
impl Contains<Location> for ParentOrParentsExecutivePlurality {
	fn contains(location: &Location) -> bool {
		matches!(location.unpack(), (1, []) | (1, [Plurality { id: BodyId::Executive, .. }]))
	}
}

pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyRecursively<DenyReserveTransferToRelayChain>,
		(
			// Allow local users to buy weight credit.
			TakeWeightCredit,
			// Expected responses are OK.
			AllowKnownQueryResponses<PolkadotXcm>,
			WithComputedOrigin<
				(
					// If the message is one that immediately attempts to pay for execution, then
					// allow it.
					AllowTopLevelPaidExecutionFrom<Everything>,
					// Parent and its pluralities (i.e. governance bodies) get free execution.
					AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
					// Subscriptions for version tracking are OK.
					AllowSubscriptionsFrom<ParentRelayOrSiblingParachains>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;
