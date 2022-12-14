// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [MSA](../pallet_msa/index.html)

use std::{num::ParseIntError, sync::Arc};
use String;

use codec::Codec;
use common_helpers::rpc::map_other_error;
#[cfg(feature = "std")]
use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	did::{Did, DidDocument, KeyType, VerificationMethod, DIDMETHOD},
	msa::{DelegatorId, MessageSourceId, ProviderId},
	node::BlockNumber,
	schema::SchemaId,
};
use did_parser::Did as DidParser;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	tracing::warn,
};
use pallet_msa_runtime_api::MsaRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

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
	) -> RpcResult<Option<Vec<SchemaId>>>;

	/// Given a DID, retrieve the MSA Id, if it is registered and active.
	#[method(name = "msa_didToMsaId")]
	fn did_to_msa_id(&self, did: Vec<u8>) -> RpcResult<Option<MessageSourceId>>;

	/// convert a given MSA Id to a DID document
	#[method(name = "msa_resolveDid")]
	fn resolve_did(&self, did: Vec<u8>) -> RpcResult<Option<String>>;
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

	/// Returns a MessageSourceId parsed from a DID, returning Error if it is not parseable or it isn't ours
	pub fn parse_msa_from_did_bytes(did_bytes: Vec<u8>) -> Result<MessageSourceId, String> {
		let maybe_did_str = String::from_utf8(did_bytes).map_err(|e| e.to_string())?;
		let (_, parsed_did) =
			DidParser::parse(maybe_did_str.as_str()).map_err(|e| e.to_string())?;
		if parsed_did.method != DIDMETHOD {
			return Err("not ours".to_string())
		}
		parsed_did.id.parse().map_err(|e: ParseIntError| e.to_string())
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

	/// A simple way to find out if a DID resolves to an MSA ID and nothing else.
	fn did_to_msa_id(&self, did: Vec<u8>) -> RpcResult<Option<MessageSourceId>> {
		match Self::parse_msa_from_did_bytes(did) {
			Ok(msa_id) => {
				let api = self.client.runtime_api();
				let at = BlockId::hash(self.client.info().best_hash);
				match api.get_public_key_count_by_msa_id(&at, msa_id) {
					Ok(key_count) => match key_count {
						0 => Ok(None),
						_ => Ok(Some(msa_id)),
					},
					Err(e) => map_rpc_result(Err(e)),
				}
			},
			Err(e) => map_other_error(e.to_string()),
		}
	}

	/// Resolves an DID to a DID document.
	fn resolve_did(&self, did: Vec<u8>) -> RpcResult<Option<String>> {
		let msa_id = Self::parse_msa_from_did_bytes(did).or_else(|e| map_other_error(e))?;

		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);

		let key_count = api
			.get_public_key_count_by_msa_id(&at, msa_id)
			.or_else(|e| map_rpc_result(Err(e)))?;
		match key_count {
			0 => Ok(None),
			_ => {
				let mut doc: DidDocument =
					DidDocument::new(Did::new(msa_id.into()), Did::new(msa_id.into()));
				// get providers for msa
				let mut capability_delegations: Vec<VerificationMethod> = vec![];
				let provider_ids = api.get_providers_for_msa_id(&at, msa_id).unwrap();
				for provider in provider_ids {
					capability_delegations.push(VerificationMethod {
						id: Did::new(u64::from(provider)),
						controller: Did::new(u64::from(provider)),
						key_type: KeyType::Sr25519,
						blockchain_account_id: 0, // TODO: blocked by Issue #418
					})
				}
				doc.capability_delegation = capability_delegations;
				// get the key that was used to sign the delegation???? or just any key, or all keys?
				// because we don't necessarily know what key the provider will use to sign batch message announcements.
				match serde_json::to_string(&doc) {
					Ok(stringified) => Ok(Some(stringified)),
					Err(e) => map_other_error(e.to_string()),
				}
			},
		}
	}
}
