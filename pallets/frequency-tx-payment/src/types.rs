// use frame_support::weights::Weight;
/// SomeDocs
pub trait GetStableWeight<RuntimeCall, Weight> {
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight>;
}
