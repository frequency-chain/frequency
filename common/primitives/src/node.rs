#![cfg_attr(not(feature = "std"), no_std)]
pub use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiAddress, MultiSignature, OpaqueExtrinsic,
};
use sp_std::{boxed::Box, vec::Vec};

use frame_support::dispatch::{DispatchError, DispatchResultWithPostInfo};

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Balance is a generic type for the balance of an account.
pub type Balance = u128;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// An index to a block.
pub type BlockNumber = u32;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Index of a transaction in the chain.
pub type Index = u32;

/// the time period in blocks that Staking Rewards are based upon
pub type Era = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// The provider of a collective action interface, for example an instance of `pallet-collective`.
pub trait ProposalProvider<AccountId, Proposal> {
	/// Add a new proposal with a threshold number of council votes.
	/// Returns a proposal length and active proposals count if successful.
	fn propose(
		who: AccountId,
		threshold: u32,
		proposal: Box<Proposal>,
	) -> Result<(u32, u32), DispatchError>;

	/// Add a new proposal with a simple majority (>50%) of council votes.
	/// Returns a proposal length and active proposals count if successful.
	fn propose_with_simple_majority(
		who: AccountId,
		proposal: Box<Proposal>,
	) -> Result<(u32, u32), DispatchError>;

	/// Get the number of proposals
	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32;
}

/// The provider for interfacing into the Utility pallet.
pub trait UtilityProvider<Origin, RuntimeCall> {
	/// Passthrough into the Utility::batch_all call
	fn batch_all(origin: Origin, calls: Vec<RuntimeCall>) -> DispatchResultWithPostInfo;
}
