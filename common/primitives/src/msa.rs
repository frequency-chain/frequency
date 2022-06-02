use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type MessageSenderId = u64;

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

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct ProviderInfo<BlockNumber> {
	pub permission: u8,
	pub expired: BlockNumber,
}

#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Provider(pub MessageSenderId);

impl From<MessageSenderId> for Provider {
	fn from(t: MessageSenderId) -> Self {
		Provider(t)
	}
}

impl From<Provider> for MessageSenderId {
	fn from(t: Provider) -> MessageSenderId {
		t.0
	}
}

pub trait AccountProvider {
	type AccountId;
	type BlockNumber;
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId>;
	fn get_provider_info_of(
		provider: Provider,
		delegator: Delegator,
	) -> Option<ProviderInfo<Self::BlockNumber>>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId, BlockNumber> {
	pub key: AccountId,
	pub msa_id: MessageSenderId,
	pub nonce: u32,
	pub expired: BlockNumber,
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
pub struct Provider(pub MessageSenderId);

impl From<MessageSenderId> for Provider {
	fn from(t: MessageSenderId) -> Self {
		Provider(t)
	}
}

impl From<Provider> for MessageSenderId {
	fn from(t: Provider) -> MessageSenderId {
		t.0
	}
}
