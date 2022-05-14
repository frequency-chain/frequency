use super::*;
use common_primitives::msa::{KeyInfoResponse, MessageSenderId};
use scale_info::TypeInfo;

use codec::{Decode, Encode};

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddKeyData {
	pub msa_id: MessageSenderId,
	pub nonce: u32,
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfo<BlockNumber> {
	pub msa_id: MessageSenderId,
	pub nonce: u32,
	pub expired: BlockNumber,
}

impl<BlockNumber> KeyInfo<BlockNumber>
where
	BlockNumber: Clone,
{
	pub fn map_to_response<AccountId>(
		&self,
		key: AccountId,
	) -> KeyInfoResponse<AccountId, BlockNumber>
	where
		AccountId: Clone,
	{
		KeyInfoResponse {
			key: key.clone(),
			msa_id: self.msa_id,
			nonce: self.nonce,
			expired: self.expired.clone(),
		}
	}
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct DelegateInfo {
	pub permission: u8,
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddDelegate {
	pub authorized_msa_id: MessageSenderId,
	pub permission: u8,
}

#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Delegator(pub MessageSenderId);

impl From<MessageSenderId> for Delegator {
	fn from(t: MessageSenderId) -> Self {
		Delegator(t)
	}
}

impl From<Delegator> for MessageSenderId {
	fn from(t: Delegator) -> MessageSenderId {
		t.0
	}
}

#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Delegate(pub MessageSenderId);

impl From<MessageSenderId> for Delegate {
	fn from(t: MessageSenderId) -> Self {
		Delegate(t)
	}
}

impl From<Delegate> for MessageSenderId {
	fn from(t: Delegate) -> MessageSenderId {
		t.0
	}
}
