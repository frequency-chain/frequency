use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_runtime::Saturating;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

// use crate::{Config};

/// migration module
pub mod migration {
	use crate::{Config, Pallet};
	use frame_support::pallet_prelude::GetStorageVersion;
	use frame_support::pallet_prelude::Weight;

	/// Migration structs and storage
	pub mod v2 {
		use crate::UnlockChunk;
		use codec::{Decode, Encode, MaxEncodedLen};
		use frame_support::BoundedVec;
		use scale_info::TypeInfo;
		use sp_core::Get;
		use sp_runtime::traits::AtLeast32BitUnsigned;
		use sp_std::fmt::Debug;

		#[derive(Default, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
		#[scale_info(skip_type_params(Balance, EpochNumber, MaxDataSize))]
		#[codec(mel_bound(MaxDataSize: MaxEncodedLen, OldStakingAccount<Balance, EpochNumber, MaxDataSize>: Encode))]
		pub struct OldStakingAccount<Balance, EpochNumber, MaxDataSize>
		where
			Balance: AtLeast32BitUnsigned + Copy + Debug + MaxEncodedLen,
			EpochNumber: AtLeast32BitUnsigned + Copy + Debug + MaxEncodedLen,
			MaxDataSize: Get<u32> + Debug,
		{
			/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
			pub active: Balance,
			/// The total amount of tokens in `active` and `unlocking`
			pub total: Balance,
			/// Unstaked balances that are thawing or awaiting withdrawal.
			pub unlocking: BoundedVec<UnlockChunk<Balance, EpochNumber>, MaxDataSize>,
		}
	}

	/// migrate StakingAccountLedger to use new StakingAccountDetailsV2
	pub fn migrate_to_v2<T: Config>() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		// let current_version = Pallet::<T>::current_storage_version();
		if onchain_version.gt(&2) {
			Weight::zero()
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}
}
