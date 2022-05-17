use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type MessageSenderId = u64;

pub trait AccountProvider {
	type AccountId;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId, BlockNumber> {
	pub key: AccountId,
	pub msa_id: MessageSenderId,
	pub nonce: u32,
	pub expired: BlockNumber,
}
