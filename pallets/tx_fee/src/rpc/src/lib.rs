use codec::{Codec, Decode};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_tx_fee_runtime_api::{FeeDetails, InclusionFee, RuntimeDispatchInfo, TxFeeRuntimeApi};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay},
};
use std::sync::Arc;

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[rpc]
pub trait MrcTxFeeApi<BlockHash, Balance> {
	#[rpc(name = "mrc_computeExtrinsicCost")]
	fn compute_extrinsic_cost(
		&self,
		encoded_xt: Bytes,
		at: Option<BlockHash>,
	) -> Result<FeeDetails<NumberOrHex>>;
}

pub struct MrcTxFeeHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MrcTxFeeHandler<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, Balance> MrcTxFeeApi<<Block as BlockT>::Hash, RuntimeDispatchInfo<Balance>>
	for MrcTxFeeHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: TxFeeRuntimeApi<Block, Balance>,
	Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + std::convert::From<u64>,
{
	fn compute_extrinsic_cost(
		&self,
		encoded_xt: Bytes,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<FeeDetails<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let encoded_len = encoded_xt.len() as u32;
		let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::DecodeError.into()),
			message: "Bad encoded extrinsic".into(),
			data: Some(format!("{:?}", e).into()),
		})?;
		let fee_details =
			api.compute_extrinsic_cost(&at, uxt, encoded_len).map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Failed to compute cost of unsigned extrinsic".into(),
				data: Some(format!("{:?}", e).into()),
			})?;

		let try_into_rpc_balance = |value: Balance| {
			value.try_into().map_err(|_| RpcError {
				code: ErrorCode::InvalidParams,
				message: format!("{} doesn't fit in NumberOrHex representation", value),
				data: None,
			})
		};
		Ok(FeeDetails {
			inclusion_fee: if let Some(inclusion_fee) = fee_details.inclusion_fee {
				Some(InclusionFee {
					base_fee: try_into_rpc_balance(inclusion_fee.base_fee)?,
					len_fee: try_into_rpc_balance(inclusion_fee.len_fee)?,
					adjusted_weight_fee: try_into_rpc_balance(inclusion_fee.adjusted_weight_fee)?,
				})
			} else {
				None
			},
			tip: Default::default(),
		})
	}
}
