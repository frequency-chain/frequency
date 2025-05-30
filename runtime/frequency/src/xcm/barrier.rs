use frame_support::traits::{ConstU32, Contains, Everything};

use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{
	AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom, DenyRecursively,
	DenyReserveTransferToRelayChain, DenyThenTry, TakeWeightCredit, TrailingSetTopicAsId,
	WithComputedOrigin,
};

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
			TakeWeightCredit,
			WithComputedOrigin<
				(
					// we constraint this one so that we limit who can execute instructions
					AllowTopLevelPaidExecutionFrom<Everything>,
					AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
					// ^^^ Parent and its exec plurality get free execution
					// Subscriptions for version tracking are OK.
					// AllowSubscriptionsFrom<ParentRelayOrSiblingParachains>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;
