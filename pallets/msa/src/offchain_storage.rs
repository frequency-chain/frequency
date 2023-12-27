use crate::{Config, Event};
pub use common_primitives::msa::MessageSourceId;
/// Offchain Storage for MSA
use common_primitives::offchain::{
	self as offchain_common, get_msa_account_lock_name, get_msa_account_storage_key_name,
	MSA_ACCOUNT_LOCK_TIMEOUT_EXPIRATION,
};
use frame_support::RuntimeDebugNoBound;
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, Time},
		Duration,
	},
	traits::One,
	Saturating,
};
use sp_std::{fmt::Debug, vec::Vec};

/// Pallet MSA lock prefix
pub const MSA_LOCK_PREFIX: &[u8] = b"pallet::msa::";

/// Block event storage prefix
pub const BLOCK_EVENT_KEY: &[u8] = b"frequency::block_event::msa::";

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Offchain index for MSA events count
pub const BLOCK_EVENT_COUNT_KEY: &[u8] = b"frequency::block_event::msa::count::";

/// Lock expiration timeout in in milli-seconds for initial data import msa pallet
pub const MSA_INITIAL_LOCK_TIMEOUT_EXPIRATION: u64 = 3000;

/// Lock expiration block for initial data import msa pallet
pub const MSA_INITIAL_LOCK_BLOCK_EXPIRATION: u32 = 20;

/// Lock name for initial data index for msa pallet
pub const MSA_INITIAL_LOCK_NAME: &[u8; 28] = b"Msa::ofw::initial-index-lock";

/// storage name for initial data import storage
pub const MSA_INITIAL_INDEXED_STORAGE_NAME: &[u8; 25] = b"Msa::ofw::initial-indexed";

/// Lock name for last processed block number events
pub const LAST_PROCESSED_BLOCK_LOCK_NAME: &[u8; 35] = b"Msa::ofw::last-processed-block-lock";

/// lst processed block storage name
pub const LAST_PROCESSED_BLOCK_STORAGE_NAME: &[u8; 30] = b"Msa::ofw::last-processed-block";

/// Lock expiration timeout in in milli-seconds for last processed block
pub const LAST_PROCESSED_BLOCK_LOCK_TIMEOUT_EXPIRATION: u64 = 5000;

/// Lock expiration block for initial data import msa pallet
pub const LAST_PROCESSED_BLOCK_LOCK_BLOCK_EXPIRATION: u32 = 2;

/// number of previous blocks to check to mitigate offchain worker skips processing any block
pub const NUMBER_OF_PREVIOUS_BLOCKS_TO_CHECK: u32 = 5u32;

/// Set offchain index value, used to store MSA Events to be process by offchain worker
pub fn set_offchain_index<V>(key: &[u8], value: V)
where
	V: Encode + Clone + Decode + Eq + Debug,
{
	offchain_common::set_offchain_index_value(key, value.encode().as_slice());
}

/// Get offchain index value, used to store MSA Events to be process by offchain worker
pub fn get_offchain_index<V>(key: &[u8]) -> Option<V>
where
	V: Encode + Clone + Decode + Eq + Debug,
{
	let value = offchain_common::get_index_value::<V>(key);
	match value {
		Ok(v) => v,
		Err(e) => {
			log::error!("Error getting offchain index value: {:?}", e);
			None
		},
	}
}

/// Remove offchain index value, used to store MSA Events to be process by offchain worker
pub fn remove_offchain_index(key: &[u8]) {
	offchain_common::remove_offchain_index_value(key);
}

/// Event type that is being offchain indexed
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebugNoBound)]
pub enum IndexedEvent<T: Config> {
	/// A new Message Service Account was created with a new MessageSourceId
	IndexedMsaCreated {
		/// The MSA for the Event
		msa_id: MessageSourceId,

		/// The key added to the MSA
		key: T::AccountId,
	},
	/// An AccountId has been associated with a MessageSourceId
	IndexedPublicKeyAdded {
		/// The MSA for the Event
		msa_id: MessageSourceId,

		/// The key added to the MSA
		key: T::AccountId,
	},
	/// An AccountId had all permissions revoked from its MessageSourceId
	IndexedPublicKeyDeleted {
		/// The MSA for the Event
		msa_id: MessageSourceId,
		/// The key no longer approved for the associated MSA
		key: T::AccountId,
	},
}

impl<T: Config> IndexedEvent<T> {
	/// maps a pallet event to indexed event type
	pub fn map(event: &Event<T>, msa_id: MessageSourceId) -> Option<Self> {
		match event {
			Event::MsaCreated { msa_id, key } =>
				Some(Self::IndexedMsaCreated { msa_id: *msa_id, key: key.clone() }),
			Event::PublicKeyAdded { msa_id, key } =>
				Some(Self::IndexedPublicKeyAdded { msa_id: *msa_id, key: key.clone() }),
			Event::PublicKeyDeleted { key } =>
				Some(Self::IndexedPublicKeyDeleted { msa_id, key: key.clone() }),
			_ => None,
		}
	}
}

/// Initializes the last_process_block value in offchain DB
pub fn init_last_processed_block<T: Config>(current_block_number: BlockNumberFor<T>) {
	let mut last_processed_block_lock = StorageLock::<'_, Time>::with_deadline(
		LAST_PROCESSED_BLOCK_LOCK_NAME,
		Duration::from_millis(LAST_PROCESSED_BLOCK_LOCK_TIMEOUT_EXPIRATION),
	);
	let _ = last_processed_block_lock.lock();
	let last_processed_block_storage =
		StorageValueRef::persistent(LAST_PROCESSED_BLOCK_STORAGE_NAME);

	// setting current_block-1 as the last processed so that we start indexing from current_block
	last_processed_block_storage
		.set(&current_block_number.saturating_sub(BlockNumberFor::<T>::one()));
}

/// Add a key into MSA offchain DB
pub fn add_key_to_offchain_msa<T: Config>(key: T::AccountId, msa_id: MessageSourceId) {
	let msa_lock_name = get_msa_account_lock_name(msa_id);
	let mut msa_lock = StorageLock::<'_, Time>::with_deadline(
		&msa_lock_name,
		Duration::from_millis(MSA_ACCOUNT_LOCK_TIMEOUT_EXPIRATION),
	);
	let _ = msa_lock.lock();
	let msa_storage_name = get_msa_account_storage_key_name(msa_id);
	let msa_storage = StorageValueRef::persistent(&msa_storage_name);

	let mut msa_keys =
		msa_storage.get::<Vec<T::AccountId>>().unwrap_or(None).unwrap_or(Vec::default());

	if !msa_keys.contains(&key) {
		msa_keys.push(key.clone());
		msa_storage.set(&msa_keys);
	} else {
		log::warn!("{:?} already added!", &key);
	}
}
