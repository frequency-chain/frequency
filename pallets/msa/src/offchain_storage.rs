use crate::{Config, Event};
pub use common_primitives::msa::MessageSourceId;
/// Offchain Storage for MSA
use common_primitives::offchain as offchain_common;
use parity_scale_codec::{Decode, Encode};
use sp_std::fmt::Debug;

use frame_support::RuntimeDebugNoBound;

/// Pallet MSA lock prefix
pub const MSA_LOCK_PREFIX: &[u8] = b"pallet::msa::";

/// Block event storage prefix
pub const BLOCK_EVENT_KEY: &[u8] = b"frequency::block_event::msa::";

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Offchain index for MSA events count
pub const BLOCK_EVENT_COUNT_KEY: &[u8] = b"frequency::block_event::msa::count::";

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
