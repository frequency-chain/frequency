// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [MSA](../pallet_msa/index.html)

use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	msa::{
		DelegationResponse, DelegatorId, KeyInfoResponse, MessageSourceId, ProviderId, SchemaGrant,
	},
	node::BlockNumber,
	offchain::get_msa_account_storage_key_name,
	schema::SchemaId,
};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	tracing::warn,
	types::{error::ErrorObjectOwned, ErrorObject},
};
use pallet_msa_runtime_api::MsaRuntimeApi;
use parity_scale_codec::{Codec, Decode};
use parking_lot::RwLock;
use rayon::prelude::*;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency MSA Custom RPC API
#[rpc(client, server)]
pub trait MsaApi<BlockHash, AccountId> {
	/// Check for a list of delegations
	/// Given a single provider, test a list of potential delegators
	/// At a given block number
	#[method(name = "msa_checkDelegations")]
	fn check_delegations(
		&self,
		delegator_msa_ids: Vec<DelegatorId>,
		provider_msa_id: ProviderId,
		block_number: BlockNumber,
		schema_id: Option<SchemaId>,
	) -> RpcResult<Vec<(DelegatorId, bool)>>;

	/// Retrieve the list of currently granted schemas given a delegator and provider pair
	#[method(name = "msa_grantedSchemaIdsByMsaId")]
	fn get_granted_schemas_by_msa_id(
		&self,
		delegator_msa_id: DelegatorId,
		provider_msa_id: ProviderId,
	) -> RpcResult<Option<Vec<SchemaGrant<SchemaId, BlockNumber>>>>;

	/// Retrieve the list of all delegations for a MsaId
	#[method(name = "msa_getAllGrantedDelegationsByMsaId")]
	fn get_all_granted_delegations_by_msa_id(
		&self,
		delegator_msa_id: DelegatorId,
	) -> RpcResult<Vec<DelegationResponse<SchemaId, BlockNumber>>>;

	/// Retrieve the list of keys for msa id
	#[method(name = "msa_getKeysByMsaId")]
	fn get_keys_by_msa_id(
		&self,
		msa_id: MessageSourceId,
	) -> RpcResult<Option<KeyInfoResponse<AccountId>>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MsaHandler<C, M, OffchainDB> {
	client: Arc<C>,
	offchain: Arc<RwLock<Option<OffchainDB>>>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M, OffchainDB> MsaHandler<C, M, OffchainDB>
where
	OffchainDB: Send + Sync,
{
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>, offchain: Option<OffchainDB>) -> Self {
		Self { client, offchain: Arc::new(RwLock::new(offchain)), _marker: Default::default() }
	}
}

/// Errors that occur on the client RPC
#[derive(Debug)]
pub enum MsaOffchainRpcError {
	/// Error acquiring lock
	ErrorAcquiringLock,
	/// Error decoding data
	ErrorDecodingData,
	/// Offchain indexing is not enabled
	OffchainIndexingNotEnabled,
}

impl From<MsaOffchainRpcError> for ErrorObjectOwned {
	fn from(e: MsaOffchainRpcError) -> Self {
		let msg = format!("{:?}", e);

		match e {
			MsaOffchainRpcError::ErrorAcquiringLock => ErrorObject::owned(1, msg, None::<()>),
			MsaOffchainRpcError::ErrorDecodingData => ErrorObject::owned(2, msg, None::<()>),
			MsaOffchainRpcError::OffchainIndexingNotEnabled =>
				ErrorObject::owned(3, msg, None::<()>),
		}
	}
}

#[async_trait]
impl<C, Block, OffchainDB, AccountId> MsaApiServer<<Block as BlockT>::Hash, AccountId>
	for MsaHandler<C, Block, OffchainDB>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MsaRuntimeApi<Block, AccountId>,
	AccountId: Codec,
	OffchainDB: sp_core::offchain::OffchainStorage + 'static,
{
	fn check_delegations(
		&self,
		delegator_msa_ids: Vec<DelegatorId>,
		provider_msa_id: ProviderId,
		block_number: BlockNumber,
		schema_id: Option<SchemaId>,
	) -> RpcResult<Vec<(DelegatorId, bool)>> {
		let at = self.client.info().best_hash;
		let results = delegator_msa_ids
			.par_iter()
			.map(|delegator_msa_id| {
				let api = self.client.runtime_api();
				// api.has_delegation returns  Result<bool, ApiError>), so _or(false) should not happen,
				// but just in case, protect against panic
				let has_delegation: bool = match api.has_delegation(
					at,
					*delegator_msa_id,
					provider_msa_id,
					block_number,
					schema_id,
				) {
					Ok(result) => result,
					Err(e) => {
						warn!("ApiError from has_delegation! {:?}", e);
						false
					},
				};
				(*delegator_msa_id, has_delegation)
			})
			.collect();
		Ok(results)
	}

	fn get_granted_schemas_by_msa_id(
		&self,
		delegator_msa_id: DelegatorId,
		provider_msa_id: ProviderId,
	) -> RpcResult<Option<Vec<SchemaGrant<SchemaId, BlockNumber>>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let runtime_api_result =
			api.get_granted_schemas_by_msa_id(at, delegator_msa_id, provider_msa_id);
		map_rpc_result(runtime_api_result)
	}

	fn get_all_granted_delegations_by_msa_id(
		&self,
		delegator_msa_id: DelegatorId,
	) -> RpcResult<Vec<DelegationResponse<SchemaId, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let runtime_api_result = api.get_all_granted_delegations_by_msa_id(at, delegator_msa_id);
		map_rpc_result(runtime_api_result)
	}

	fn get_keys_by_msa_id(
		&self,
		msa_id: MessageSourceId,
	) -> RpcResult<Option<KeyInfoResponse<AccountId>>> {
		let msa_key = get_msa_account_storage_key_name(msa_id);
		let reader = self.offchain.try_read().ok_or(MsaOffchainRpcError::ErrorAcquiringLock)?;
		let raw: Option<Bytes> = reader
			.as_ref()
			.ok_or(MsaOffchainRpcError::OffchainIndexingNotEnabled)?
			.get(sp_offchain::STORAGE_PREFIX, &msa_key)
			.map(Into::into);
		if let Some(rr) = raw {
			let inside = rr.0;
			let keys = Vec::<AccountId>::decode(&mut &inside[..])
				.map_err(|_| MsaOffchainRpcError::ErrorDecodingData)?;
			return Ok(Some(KeyInfoResponse { msa_id, msa_keys: keys }))
		}
		Ok(None)
	}
}
