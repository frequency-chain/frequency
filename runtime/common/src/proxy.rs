use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use sp_core::RuntimeDebug;

// Proxy Pallet Config
/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	DecodeWithMemTracking,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	Any = 0,
	/// Can execute any call that does not transfer funds or assets.
	NonTransfer = 1,
	Governance = 2,
	Staking = 3,
	// Skip: SudoBalances = 4, IdentityJudgement = 5,
	/// Proxy with the ability to reject time-delay proxy announcements.
	CancelProxy = 6,
	// Skip: Auction = 7, NominationPools = 8,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
