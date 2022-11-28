// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [MSA](../pallet_msa/index.html)

use std::sync::Arc;
use String;

use codec::Codec;
use did_parser::Did as DidParser;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	tracing::warn,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	did::DidDocument,
	msa::{DelegatorId, MessageSourceId, ProviderId},
	node::BlockNumber,
	schema::SchemaId,
};

use common_primitives::did::Did;

use pallet_msa_runtime_api::MsaRuntimeApi;

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
	) -> RpcResult<Option<Vec<SchemaId>>>;

	/// Given a DID, retrieve the MSA Id, if it is registered and active.
	#[method(name = "msa_didToMsaId")]
	fn did_to_msa_id(&self, did: Vec<u8>) -> RpcResult<Option<MessageSourceId>>;

	/// convert a given MSA Id to a DID document
	#[method(name = "msa_idToDidDocument")]
	fn msa_id_to_did_document(&self, msa_id: MessageSourceId) -> RpcResult<Option<String>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MsaHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MsaHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, AccountId> MsaApiServer<<Block as BlockT>::Hash, AccountId> for MsaHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MsaRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
	// fn get_msa_keys(&self, msa_id: MessageSourceId) -> RpcResult<Vec<KeyInfoResponse<AccountId>>> {
	// 	let api = self.client.runtime_api();
	// 	let at = BlockId::hash(self.client.info().best_hash);
	// 	let runtime_api_result = api.get_msa_keys(&at, msa_id);
	// 	map_rpc_result(runtime_api_result)
	// }

	fn check_delegations(
		&self,
		delegator_msa_ids: Vec<DelegatorId>,
		provider_msa_id: ProviderId,
		block_number: BlockNumber,
		schema_id: Option<SchemaId>,
	) -> RpcResult<Vec<(DelegatorId, bool)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);

		Ok(delegator_msa_ids
			.iter() // TODO: Change back to par_iter() which has borrow panic GitHub Issue: #519
			.map(|&delegator_msa_id| {
				// api.has_delegation returns  Result<bool, ApiError>), so _or(false) should not happen,
				// but just in case, protect against panic
				let has_delegation: bool = match api.has_delegation(
					&at,
					delegator_msa_id,
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
				(delegator_msa_id, has_delegation)
			})
			.collect())
	}

	fn get_granted_schemas_by_msa_id(
		&self,
		delegator_msa_id: DelegatorId,
		provider_msa_id: ProviderId,
	) -> RpcResult<Option<Vec<SchemaId>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result =
			api.get_granted_schemas_by_msa_id(&at, delegator_msa_id, provider_msa_id);
		map_rpc_result(runtime_api_result)
	}

	// This returns some stuff
	fn did_to_msa_id(&self, did: Vec<u8>) -> RpcResult<Option<MessageSourceId>> {
		let result = String::from_utf8(did);
		match result {
			Err(_) => Ok(None),
			Ok(did_string) => {
				let res = DidParser::parse(&did_string);
				if res.is_err() {
					return Ok(None)
				}
				let (_, did) = res.unwrap();
				let api = self.client.runtime_api();
				let at = BlockId::hash(self.client.info().best_hash);

				// TODO: remove unwraps & handle errors
				let msa_id: MessageSourceId = did.id.clone().parse().unwrap();
				let key_count = api.get_public_key_count_by_msa_id(&at, msa_id).unwrap();
				match key_count {
					0 => Ok(None),
					_ => Ok(Some(msa_id)),
				}
			},
		}
	}

	fn msa_id_to_did_document(&self, msa_id: MessageSourceId) -> RpcResult<Option<String>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let key_count = api.get_public_key_count_by_msa_id(&at, msa_id).unwrap();
		match key_count {
			0 => Ok(None),
			_ => {
				let doc: DidDocument =
					DidDocument::new(Did::new(msa_id.into()), Did::new(msa_id.into()));
				let result = serde_json::to_string(&doc).unwrap();
				Ok(Some(result))
			},
		}
	}
}
