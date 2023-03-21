use frame_support::weights::Weight;

pub trait WeightInfo {
	fn claim_handle() -> Weight;
}
