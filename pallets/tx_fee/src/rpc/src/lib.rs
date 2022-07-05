use codec::{Codec, Decode};
use jsonrpsee::{
	core::{async_trait, Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
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

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[rpc(client, server)]
pub trait FrequencyTxFeeApi<BlockHash, Balance> {
	#[method(name = "frequency_computeExtrinsicCost")]
	fn compute_extrinsic_cost(
		&self,
		encoded_xt: Bytes,
		at: Option<BlockHash>,
	) -> RpcResult<FeeDetails<NumberOrHex>>;
}

pub struct FrequencyTxFeeHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> FrequencyTxFeeHandler<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, Balance>
	FrequencyTxFeeApiServer<<Block as BlockT>::Hash, RuntimeDispatchInfo<Balance>>
	for FrequencyTxFeeHandler<C, Block>
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
	) -> RpcResult<FeeDetails<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let encoded_len = encoded_xt.len() as u32;
		let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| {
			RpcError::Call(CallError::Custom(ErrorObject::owned(
				Error::DecodeError.into(),
				"Bad encoded extrinsic",
				Some(format!("{:?}", e)),
			)))
		})?;
		let fee_details = api.compute_extrinsic_cost(&at, uxt, encoded_len).map_err(|e| {
			RpcError::Call(CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Failed to compute cost of unsigned extrinsic",
				Some(format!("{:?}", e)),
			)))
		})?;

		let try_into_rpc_balance = |value: Balance| {
			value.try_into().map_err(|_| {
				RpcError::Call(CallError::Custom(ErrorObject::owned(
					ErrorCode::InvalidParams.code(),
					format!("{} doesn't fit in NumberOrHex representation", value),
					None::<()>,
				)))
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
