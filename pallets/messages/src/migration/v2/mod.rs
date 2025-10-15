use crate::{Config, MessageIndex, Pallet};
use common_primitives::{msa::MessageSourceId, schema::SchemaId};
use core::fmt::Debug;
use frame_support::{pallet_prelude::*, storage_alias};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::codec::{Decode, Encode};

/// Storage for messages in v1 and lower
/// - Key: (block_number, schema_id, message_index)
/// - Value: Message
#[storage_alias]
pub(crate) type MessagesV2<T: Config> = StorageNMap<
	Pallet<T>,
	(
		storage::Key<Twox64Concat, BlockNumberFor<T>>,
		storage::Key<Twox64Concat, SchemaId>,
		storage::Key<Twox64Concat, MessageIndex>,
	),
	Message<<T as Config>::MessagesMaxPayloadSizeBytes>,
	OptionQuery,
>;

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
