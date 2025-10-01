use frame_support::pallet_prelude::*;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Old provider registry entry structure.
#[derive(MaxEncodedLen, TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct ProviderRegistryEntry<T>
where
	T: Get<u32>,
{
	/// The provider's name
	pub provider_name: BoundedVec<u8, T>,
}
