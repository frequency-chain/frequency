pub type MessageSenderId = u64;

pub trait AccountProvider {
	type AccountId;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSenderId>;
}
