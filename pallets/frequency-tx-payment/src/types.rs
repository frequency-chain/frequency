use sp_std::vec::Vec;

/// Gets stable weights for a capacity Call
pub trait GetStableWeight<RuntimeCall, Weight> {
	/// Get stable weights for Call
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight>;

	/// Get inner calls from a Call if any exist,
	/// e.g. in case of `pay_with_capacity` and `pay_with_capacity_batch_all`
	fn get_inner_calls(outer_call: &RuntimeCall) -> Option<Vec<&RuntimeCall>>;
}
