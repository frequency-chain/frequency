use crate::{Config, MessageIndex, Pallet};
use common_primitives::{msa::MessageSourceId, schema::SchemaId};
use core::fmt::Debug;
use frame_support::{pallet_prelude::*, storage_alias};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::codec::{Decode, Encode};

// Type aliases for readability
#[allow(type_alias_bounds)]
pub(crate) type K1Type<T: Config> = BlockNumberFor<T>;
#[allow(type_alias_bounds)]
pub(crate) type K2Type = SchemaId;
#[allow(type_alias_bounds)]
pub(crate) type K3Type = MessageIndex;

/// Storage for messages in v2 and lower
/// - Key: (block_number, schema_id, message_index)
/// - Value: Message
#[storage_alias]
pub(crate) type MessagesV2<T: Config> = StorageNMap<
	Pallet<T>,
	(
		storage::Key<Twox64Concat, K1Type<T>>,
		storage::Key<Twox64Concat, K2Type>,
		storage::Key<Twox64Concat, K3Type>,
	),
	Message<<T as Config>::MessagesMaxPayloadSizeBytes>,
	OptionQuery,
>;

/// Ephemeral storage key for tracking the completion status of the migration
/// in order to perform the final step and for try-runtime. MUST be killed at
/// the end of the migration!
#[storage_alias]
pub(crate) type DoneV3Migration<T: Config> = StorageValue<Pallet<T>, bool, ValueQuery>;

/// A single message type definition for V2 storage
#[derive(Clone, Default, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxDataSize))]
#[codec(mel_bound(MaxDataSize: MaxEncodedLen))]
pub struct Message<MaxDataSize>
where
	MaxDataSize: Get<u32> + Debug,
{
	///  Data structured by the associated schema's model.
	pub payload: BoundedVec<u8, MaxDataSize>,
	/// Message source account id of the Provider. This may be the same id as contained in `msa_id`,
	/// indicating that the original source MSA is acting as its own provider. An id differing from that
	/// of `msa_id` indicates that `provider_msa_id` was delegated by `msa_id` to send this message on
	/// its behalf.
	pub provider_msa_id: MessageSourceId,
	///  Message source account id (the original source).
	pub msa_id: Option<MessageSourceId>,
}
