use sp_std::{marker::PhantomData, result::Result};

use xcm_executor::traits::ShouldExecute;
use frame_support::traits::Get;
use xcm::latest::{
	Instruction::{self, *},
	MultiLocation,
	Weight,
	Xcm,
};


/// A derivative barrier, which scans the first `MaxPrefixes` instructions for origin-alterers and
/// then evaluates `should_execute` of the `InnerBarrier` based on the remaining instructions and
/// the newly computed origin.
///
/// This effectively allows for the possibility of distinguishing an origin which is acting as a
/// router for its derivative locations (or as a bridge for a remote location) and an origin which
/// is actually trying to send a message for itself. In the former case, the message will be
/// prefixed with origin-mutating instructions.
///
/// Any barriers which should be interpreted based on the computed origin rather than the original
/// message origin should be subject to this. This is the case for most barriers since the
/// effective origin is generally more important than the routing origin. Any other barriers, and
/// especially those which should be interpreted only the routing origin should not be subject to
/// this.
///
/// E.g.
/// ```nocompile
/// type MyBarrier = (
/// 	TakeWeightCredit,
/// 	AllowTopLevelPaidExecutionFrom<DirectCustomerLocations>,
/// 	WithComputedOrigin<(
/// 		AllowTopLevelPaidExecutionFrom<DerivativeCustomerLocations>,
/// 		AllowUnpaidExecutionFrom<ParentLocation>,
/// 		AllowSubscriptionsFrom<AllowedSubscribers>,
/// 		AllowKnownQueryResponses<TheResponseHandler>,
/// 	)>,
/// );
/// ```
///
/// In the above example, `AllowUnpaidExecutionFrom` appears once underneath
/// `WithComputedOrigin`. This is in order to distinguish between messages which are notionally
/// from a derivative location of `ParentLocation` but that just happened to be sent via
/// `ParentLocaction` rather than messages that were sent by the parent.
///
/// Similarly `AllowTopLevelPaidExecutionFrom` appears twice: once inside of `WithComputedOrigin`
/// where we provide the list of origins which are derivative origins, and then secondly outside
/// of `WithComputedOrigin` where we provide the list of locations which are direct origins. It's
/// reasonable for these lists to be merged into one and that used both inside and out.
///
/// Finally, we see `AllowSubscriptionsFrom` and `AllowKnownQueryResponses` are both inside of
/// `WithComputedOrigin`. This means that if a message begins with origin-mutating instructions,
/// then it must be the finally computed origin which we accept subscriptions or expect a query
/// response from. For example, even if an origin appeared in the `AllowedSubscribers` list, we
/// would ignore this rule if it began with origin mutators and they changed the origin to something
/// which was not on the list.
pub struct WithComputedOrigin<InnerBarrier, MaxPrefixes>(
	PhantomData<(InnerBarrier, MaxPrefixes)>,
);
impl<
		InnerBarrier: ShouldExecute,
		MaxPrefixes: Get<u32>,
	> ShouldExecute for WithComputedOrigin<InnerBarrier, MaxPrefixes>
{
	fn should_execute<Call>(
		origin: &MultiLocation,
		instructions: &mut Xcm<Call>,
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), ()> {
		let mut actual_origin: MultiLocation = origin.clone();
		let mut skipped = 0;
		// NOTE: We do not check the validity of `UniversalOrigin` here, meaning that a malicious
		// origin could place a `UniversalOrigin` in order to spoof some location which gets free
		// execution. This technical could get it past the barrier condition, but the execution
		// would instantly fail since the first instruction would cause an error with the
		// invalid UniversalOrigin.
		// let mut iter = instructions.0;
		while skipped < MaxPrefixes::get() as usize {
			match instructions.0.get(skipped) {
				// Some(UniversalOrigin(new_global)) => {
				// 	// Note the origin is *relative to local consensus*! So we need to escape local
				// 	// consensuxxs with the `parents` before diving in into the `universal_location`.
				// 	actual_origin = X1(*new_global).relative_to(&LocalUniversal::get());
				// },
				Some(DescendOrigin(j)) => {
					actual_origin.append_with(j.clone()).map_err(|_| ())?;
				},
				_ => break,
			}
			skipped += 1;
		}

		let mut xcm = instructions.clone();
		xcm.0 = instructions.0[skipped..].to_vec();


		InnerBarrier::should_execute(
			&actual_origin,
			 &mut xcm,
			max_weight,
			weight_credit,
		)
	}
}
