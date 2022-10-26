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

#[cfg(test)]
mod tests {
	use super::*;

	// It is enough to test that we are getting into the correct match statement.

	#[test]
	fn maps_ok_ok_to_ok() {
		let result = map_rpc_result(Ok(Ok(0)));
		assert!(result.is_ok());
		assert_eq!(0, result.unwrap());
	}

	#[test]
	fn maps_ok_dispatch_err() {
		let dispach_err = sp_runtime::ModuleError { index: 0, error: [0, 0, 0, 0], message: None };
		let result = map_rpc_result::<u64>(Ok(Err(sp_runtime::DispatchError::Module(dispach_err))));
		assert!(result.is_err());
		let str = format!("{:?}", result.err().unwrap());
		assert!(str.contains("Dispatch Module Error"), "Did not find in: {:?}", str);
	}

	#[test]
	fn maps_ok_dispatch_err_to_call_err() {
		let result = map_rpc_result::<u64>(Ok(Err(sp_runtime::DispatchError::Other("test"))));
		assert!(result.is_err());
		let str = format!("{:?}", result.err().unwrap());
		assert!(str.contains("Dispatch Error"), "Did not find in: {:?}", str);
	}

	#[test]
	fn maps_err_to_api_err() {
		let result = map_rpc_result::<u64>(Err(ApiError::StateBackendIsNotTrie));
		assert!(result.is_err());
		let str = format!("{:?}", result.err().unwrap());
		assert!(str.contains("Api Error"), "Did not find in: {:?}", str);
	}
}
