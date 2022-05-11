use common_primitives::msa::MessageSenderId;
use scale_info::TypeInfo;

use codec::{Decode, Encode};

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddKeyData {
	pub msa_id: MessageSenderId,
	pub nonce: u32,
}
