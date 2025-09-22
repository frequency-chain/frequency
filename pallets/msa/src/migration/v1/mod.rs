use crate::Config;
use common_primitives::msa::ProviderId;
use frame_support::{pallet_prelude::*, storage_alias};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Storage alias for the old `ProviderToRegistryEntry` storage format used in migration v1.
///
/// This maps:
/// - `ProviderId` → a legacy [`ProviderRegistryEntry`] bounded by `MaxProviderNameSize`.
///
/// # Type Parameters
/// * `T` - The pallet's configuration trait, which supplies `MaxProviderNameSize`.
#[storage_alias]
#[allow(missing_docs)]
pub type ProviderToRegistryEntry<T: Config> = StorageMap<
	crate::Pallet<T>,
	Twox64Concat,
	ProviderId,
	ProviderRegistryEntry<<T as Config>::MaxProviderNameSize>,
	OptionQuery,
>;

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
