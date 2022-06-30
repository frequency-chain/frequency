use core::result::Result as CoreResult;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use sp_api::ApiError;
use sp_runtime::DispatchError;

pub fn map_rpc_result<T>(
	response: CoreResult<CoreResult<T, DispatchError>, ApiError>,
) -> Result<T> {
	match response {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(DispatchError::Module(e))) => Err(RpcError {
			code: ErrorCode::ServerError(100), // No real reason for this value
			message: format!("Dispatch Error Module:{} error:{}", e.index, e.error).into(),
			data: Some(e.message.unwrap_or_default().into()),
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
