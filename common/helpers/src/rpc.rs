use core::result::Result as CoreResult;
use jsonrpsee::{
	core::RpcResult,
	types::error::{ErrorCode, ErrorObject},
};
use sp_api::ApiError;

/// Converts CoreResult to Result for RPC calls
pub fn map_rpc_result<T>(response: CoreResult<T, ApiError>) -> RpcResult<T> {
	match response {
		Ok(res) => Ok(res),
		Err(e) => Err(ErrorObject::owned(
			ErrorCode::ServerError(300).code(), // No real reason for this value
			"Api Error",
			Some(format!("{:?}", e)),
		)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// It is enough to test that we are getting into the correct match statement.

	#[test]
	fn maps_ok_to_ok() {
		let result = map_rpc_result(Ok(0));
		assert!(result.is_ok());
		assert_eq!(0, result.unwrap());
	}

	#[test]
	fn maps_err_to_api_err() {
		let result = map_rpc_result::<u64>(Err(ApiError::StateBackendIsNotTrie));
		assert!(result.is_err());
		let str = format!("{:?}", result.err().unwrap());
		assert!(str.contains("Api Error"), "Did not find in: {:?}", str);
	}
}
