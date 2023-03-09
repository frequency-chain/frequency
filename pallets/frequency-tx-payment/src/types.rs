/// Gets stable weights for a capacity Call
pub trait GetStableWeight<RuntimeCall, Weight> {
	/// Get stable weights for Call
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight>;
}
