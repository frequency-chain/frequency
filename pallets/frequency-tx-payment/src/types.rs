// use frame_support::weights::Weight;
/// SomeDocs
pub trait GetStableWeight<RuntimeCall> {
    type Weight;

    fn get_stable_weight(call: &RuntimeCall) -> Option<Self::Weight>;
}