use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Get, BoundedVec};
use orml_utilities::OrderedSet;
use scale_info::TypeInfo;

/// Extending OrderSet and implement struct OrderedSetExt
#[derive(TypeInfo, Clone, Decode, Encode, PartialEq, Eq, MaxEncodedLen, Default)]
#[scale_info(skip_type_params(T, S))]
pub struct OrderedSetExt<
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq,
	S: Get<u32>,
>(pub OrderedSet<T, S>);

impl<T, S> OrderedSetExt<T, S>
where
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq + core::fmt::Debug,
	S: Get<u32>,
{
	/// Create a new empty set
	pub fn new() -> Self {
		Self(OrderedSet::<T, S>::new())
	}
	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(bv: BoundedVec<T, S>) -> Self {
		Self(OrderedSet::<T, S>::from(bv))
	}
}

impl<T, S> core::fmt::Debug for OrderedSetExt<T, S>
where
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq + core::fmt::Debug,
	S: Get<u32>,
{
	fn fmt(&self, _f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}
