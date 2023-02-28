
use pallet_messages::Call as MessagesCall;
use pallet_msa::Call as MsaCall;
pub struct CapacityTxCalls;
impl Contains<RuntimeCall> for CapacityTxCalls {
	fn contains(call: &RuntimeCall) -> bool {
		{
			match call {
				RuntimeCall::Msa(MsaCall::add_public_key_to_msa { .. }) => true,
				RuntimeCall::Msa(MsaCall::create_sponsored_account_with_delegation { .. }) => true,
				RuntimeCall::Msa(MsaCall::grant_delegation { .. }) => true,
				RuntimeCall::Messages(MessagesCall::add_ipfs_message { .. }) => true,
				RuntimeCall::Messages(MessagesCall::add_onchain_message { .. }) => true,
				_ => false,
			}
		}
	}
}
