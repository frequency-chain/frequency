use codec::Codec;
use common_primitives::weight_to_fees::WeightToFee;
use core::result::Result as CoreResult;
use frame_support::weights::WeightToFeePolynomial;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use pallet_transaction_payment_rpc_runtime_api::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay},
	DispatchError,
};
use sp_std::vec::Vec;
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
pub trait SchemasApi<BlockHash, AccountId, ResponseType> {
	#[rpc(name = "schemas_getLatestSchemaId")]
	fn get_latest_schema_id(&self, at: Option<BlockHash>) -> Result<u16>;
	#[rpc(name = "schemas_calculateSchemaFee")]
	fn get_schema_registration_fee(
		&self,
		at: Option<BlockHash>,
		schema: Vec<u8>,
	) -> Result<FeeDetails<NumberOrHex>>;
}

pub struct SchemasHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemasHandler<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, Balance>
	SchemasApi<<Block as BlockT>::Hash, AccountId, RuntimeDispatchInfo<Balance>>
	for SchemasHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: SchemasRuntimeApi<Block, AccountId, Balance>,
	AccountId: Codec,
	Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + std::convert::From<u64>,
{
	fn get_latest_schema_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u16> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_api_result = api.get_latest_schema_id(&at);
		map_result(schema_api_result)
	}

	fn get_schema_registration_fee(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		schema: Vec<u8>,
	) -> Result<FeeDetails<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_weight_res = api.calculate_schema_cost(&at, schema);
		let schema_weight = match schema_weight_res {
			Ok(weight) => weight,
			Err(err) =>
				return Err(RpcError {
					code: ErrorCode::ServerError(1),
					message: "Unable to calculate weight".into(),
					data: Some(format!("{:?}", err).into()),
				}),
		};
		let unadjusted_schema_fee = WeightToFee::calc(&schema_weight);
		let try_into_rpc_balance = |value: Balance| {
			value.try_into().map_err(|_| RpcError {
				code: ErrorCode::InvalidParams,
				message: format!("{} doesn't fit in NumberOrHex representation", value),
				data: None,
			})
		};
		// TODO Issue #77: Check what fee calculation should be like
		let fixed_len_fee = 0u64.into();
		let tip = 0u64.into();
		let base_fee = 0u64.into();
		let fee_return = FeeDetails {
			inclusion_fee: Some(InclusionFee {
				base_fee: try_into_rpc_balance(base_fee)?,
				len_fee: try_into_rpc_balance(fixed_len_fee)?,
				adjusted_weight_fee: try_into_rpc_balance(unadjusted_schema_fee.into())?,
			}),
			tip,
		};
		Ok(fee_return)
	}
}

fn map_result<T>(response: CoreResult<CoreResult<T, DispatchError>, ApiError>) -> Result<T> {
	match response {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(DispatchError::Module(error))) => Err(RpcError {
			code: ErrorCode::ServerError(100), // No real reason for this value
			message: format!("Dispatch Error Module:{} error:{}", error.index, error.error).into(),
			data: Some(error.message.unwrap_or_default().into()),
		}),
		Ok(Err(e)) => Err(RpcError {
			code: ErrorCode::ServerError(200), // No real reason for this value
			message: "Dispatch Error".into(),
			data: Some(format!("{:?}", e).into()),
		}),
		Err(e) => Err(RpcError {
			code: ErrorCode::ServerError(300), // No real reason for this value
			message: "Api Error".into(),
			data: Some(format!("{:?}", e).into()),
		}),
	}
}
