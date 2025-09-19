// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [User-Handles](../pallet_handles/index.html)

use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	handles::{
		BaseHandle, DisplayHandle, HandleResponse, PresumptiveSuffixesResponse,
		DEFAULT_SUFFIX_COUNT, MAX_SUFFIXES_COUNT,
	},
	msa::MessageSourceId,
};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::{error::ErrorObjectOwned, ErrorObject},
};
use pallet_handles_runtime_api::HandlesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Handles Custom RPC API
#[rpc(client, server)]
pub trait HandlesApi<BlockHash> {
	/// retrieve `HandleResponse` for a given `MessageSourceId`
	#[method(name = "handles_getHandleForMsa")]
	fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> RpcResult<Option<HandleResponse>>;

	/// retrieve next `n` suffixes for a given handle
	#[method(name = "handles_getNextSuffixes")]
	fn get_next_suffixes(
		&self,
		base_handle: String,
		count: Option<u16>,
	) -> RpcResult<PresumptiveSuffixesResponse>;

	/// retrieve `MessageSourceId` for a given handle
	#[method(name = "handles_getMsaForHandle")]
	fn get_msa_for_handle(&self, display_handle: String) -> RpcResult<Option<MessageSourceId>>;

	/// validate a handle
	#[method(name = "handles_validateHandle")]
	fn validate_handle(&self, base_handle: String) -> RpcResult<bool>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct HandlesHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> HandlesHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Errors that occur on the client RPC
#[derive(Debug)]
pub enum HandlesRpcError {
	/// InvalidHandle
	InvalidHandle,
}

impl From<HandlesRpcError> for ErrorObjectOwned {
	fn from(e: HandlesRpcError) -> Self {
		let msg = format!("{e:?}");

		match e {
			HandlesRpcError::InvalidHandle => ErrorObject::owned(1, msg, None::<()>),
		}
	}
}

#[async_trait]
impl<C, Block> HandlesApiServer<<Block as BlockT>::Hash> for HandlesHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: HandlesRuntimeApi<Block>,
{
	fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> RpcResult<Option<HandleResponse>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let result = api.get_handle_for_msa(at, msa_id);
		map_rpc_result(result)
	}

	fn get_next_suffixes(
		&self,
		base_handle: String,
		count: Option<u16>,
	) -> RpcResult<PresumptiveSuffixesResponse> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let base_handle: BaseHandle = base_handle
			.into_bytes()
			.try_into()
			.map_err(|_| HandlesRpcError::InvalidHandle)?;
		let max_count = MAX_SUFFIXES_COUNT;
		let count = count.unwrap_or(DEFAULT_SUFFIX_COUNT).min(max_count);
		let suffixes_result = api.get_next_suffixes(at, base_handle, count);
		map_rpc_result(suffixes_result)
	}

	fn get_msa_for_handle(&self, display_handle: String) -> RpcResult<Option<MessageSourceId>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let handle: DisplayHandle = display_handle
			.into_bytes()
			.try_into()
			.map_err(|_| HandlesRpcError::InvalidHandle)?;
		let result = api.get_msa_for_handle(at, handle);
		map_rpc_result(result)
	}

	fn validate_handle(&self, base_handle: String) -> RpcResult<bool> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let base_handle: BaseHandle = base_handle.into_bytes().try_into().unwrap_or_default();
		let result = api.validate_handle(at, base_handle);
		map_rpc_result(result)
	}
}
