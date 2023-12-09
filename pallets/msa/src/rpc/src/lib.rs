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
	msa::{DelegatorId, ProviderId, SchemaGrant},
	node::{BlockNumber},
	schema::SchemaId,
};
use parity_scale_codec::Decode;
use parity_scale_codec::Codec;
use parking_lot::RwLock;
use common_primitives::msa::{KeyInfoResponse, MessageSourceId};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	tracing::warn,
};
use sp_core::Bytes;
use pallet_msa_runtime_api::MsaRuntimeApi;
use rayon::prelude::*;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::{sync::Arc};

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

	/// Retrieve the list of keys for msa id
	#[method(name = "msa_getKeysByMsaId")]
	fn get_keys_by_msa_id(
		&self,
		msa_id: MessageSourceId,
	) -> RpcResult<Option<Vec<KeyInfoResponse<AccountId>>>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MsaHandler<C, M, OffchainDB> {
	client: Arc<C>,
	offchain: Arc<RwLock<Option<OffchainDB>>>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M, OffchainDB> MsaHandler<C, M, OffchainDB> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>, offchain: Option<OffchainDB>) -> Self {
		Self { client, offchain: Arc::new(RwLock::new(offchain)), _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, OffchainDB, AccountId> MsaApiServer<<Block as BlockT>::Hash, AccountId> for MsaHandler<C, Block, OffchainDB>
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

	fn get_keys_by_msa_id(
		&self,
		msa_id: MessageSourceId,
	) -> RpcResult<Option<Vec<KeyInfoResponse<AccountId>>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		let key = vec![77u8, 115, 97, 58, 58, 111, 102, 119, 58, 58, 107, 101, 121, 115, 58, 58, 49, 48, 48];
		let reader = self.offchain.read();
		let raw :Option<Bytes> = reader.as_ref().unwrap().get(sp_offchain::STORAGE_PREFIX, &key).map(Into::into);
		if let Some(rr) = raw {
			let inside = rr.0;
			let decoded = Vec::<AccountId>::decode(&mut &inside[..]).unwrap();
			let res: Vec<KeyInfoResponse<AccountId>> = decoded.into_iter()
				.map(|account_id| KeyInfoResponse { msa_id, key: account_id })
				.collect();

			return RpcResult::Ok(Some(res))
		}

		let runtime_api_result = api.offchain_get_keys_by_msa_id(at, msa_id);
		map_rpc_result(runtime_api_result)
	}
}
