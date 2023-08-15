// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [MSA](../pallet_msa/index.html)

use codec::Codec;
use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	msa::{DelegatorId, ProviderId, SchemaGrant},
	node::BlockNumber,
	schema::SchemaId,
};

use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	tracing::warn,
};
use pallet_msa_runtime_api::MsaRuntimeApi;
use rayon::prelude::*;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Capacity Custom RPC API
#[rpc(client, server)]
pub trait CapacityApi<BlockHash, AccountId> {

}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct CapacityHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> CapacityHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, AccountId> CapacityApiServer<<Block as BlockT>::Hash, AccountId> for CapacityHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: CapacityRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{

}
