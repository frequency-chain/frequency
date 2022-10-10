use core::result::Result as CoreResult;
use jsonrpsee::{
	core::{Error as RpcError, RpcResult},
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::ApiError;
use sp_runtime::DispatchError;

/// Converts CoreResult to Result for RPC calls
/// # Arguments
/// * `response` - The response to map to an RPC response
/// # Returns
/// * `Result<T>` The RPC formatted response for JSON
pub fn map_rpc_result<T>(
	response: CoreResult<CoreResult<T, DispatchError>, ApiError>,
) -> RpcResult<T> {
	match response {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(DispatchError::Module(e))) => {
			Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
				ErrorCode::ServerError(100).code(), // No real reason for this value
				"Dispatch Module Error",
				Some(format!("{:?}", e)),
			))))
		},
		Ok(Err(e)) => Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
			ErrorCode::ServerError(200).code(), // No real reason for this value
			"Dispatch Error",
			Some(format!("{:?}", e)),
		)))),
		Err(e) => Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
			ErrorCode::ServerError(300).code(), // No real reason for this value
			"Api Error",
			Some(format!("{:?}", e)),
		)))),
	}
}
