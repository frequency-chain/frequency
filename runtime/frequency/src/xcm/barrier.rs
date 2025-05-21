use frame_support::{
	traits::{ConstU32, Contains, Everything}
};

// use frame_support::{
// 	pallet_prelude::Get,
// 	parameter_types,
// 	traits::{ConstU32, ConstU8, Contains, ContainsPair, Disabled, Everything, Nothing},
// 	weights::Weight,
// };


use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{
	DenyRecursively, DenyReserveTransferToRelayChain, DenyThenTry, TakeWeightCredit,
	WithComputedOrigin, TrailingSetTopicAsId, AllowTopLevelPaidExecutionFrom,
	AllowExplicitUnpaidExecutionFrom,
};

use crate::xcm::constants::UniversalLocation;

// parameter_types! {
//     pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into())].into();
// }

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