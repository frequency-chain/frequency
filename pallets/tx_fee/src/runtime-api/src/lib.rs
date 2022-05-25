#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use sp_runtime::traits::MaybeDisplay;

sp_api::decl_runtime_apis! {
	pub trait TxFeeRuntimeApi<Balance> where
	Balance: Codec + MaybeDisplay,
	{
		fn compute_extrinsic_cost(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
	}
}
