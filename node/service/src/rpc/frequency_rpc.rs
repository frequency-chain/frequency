// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use common_helpers::rpc::map_rpc_result;
use common_primitives::rpc::RpcEvent;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::ErrorObject,
};
use parity_scale_codec::{Codec, Decode, Encode};
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{AtLeast32Bit, Block as BlockT, One};
use std::sync::Arc;
use substrate_frame_rpc_system::AccountNonceApi;
use system_runtime_api::AdditionalRuntimeApi;

/// This is an upper limit to restrict the number of returned nonce holes to eliminate a potential
/// attack vector
const MAX_RETURNED_MISSING_NONCE_SIZE: usize = 1000;
/// Frequency MSA Custom RPC API
#[rpc(client, server)]
pub trait FrequencyRpcApi<B: BlockT, AccountId, Nonce> {
	/// gets the events for a block hash
	#[method(name = "frequency_getEvents")]
	fn get_events(&self, at: B::Hash) -> RpcResult<Vec<RpcEvent>>;
	/// returns a list of missing nonce values from Future transaction pool.
	#[method(name = "frequency_getMissingNonceValues")]
	fn get_missing_nonce_values(&self, account: AccountId) -> RpcResult<Vec<Nonce>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct FrequencyRpcHandler<P: TransactionPool, C, M> {
	client: Arc<C>,
	pool: Arc<P>,
	_marker: std::marker::PhantomData<M>,
}

impl<P: TransactionPool, C, M> FrequencyRpcHandler<P, C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
		Self { client, pool, _marker: Default::default() }
	}
}

#[async_trait]
impl<P, C, Block, AccountId, Nonce> FrequencyRpcApiServer<Block, AccountId, Nonce>
	for FrequencyRpcHandler<P, C, Block>
where
	Block: BlockT,
	C: HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C::Api: AdditionalRuntimeApi<Block>,
	C::Api: AccountNonceApi<Block, AccountId, Nonce>,
	P: TransactionPool + 'static,
	AccountId: Clone + Codec,
	Nonce: Clone + Encode + Decode + AtLeast32Bit + 'static,
{
	fn get_events(&self, at: <Block as BlockT>::Hash) -> RpcResult<Vec<RpcEvent>> {
		let api = self.client.runtime_api();
		return map_rpc_result(api.get_events(at))
	}

	fn get_missing_nonce_values(&self, account: AccountId) -> RpcResult<Vec<Nonce>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;

		let nonce = api
			.account_nonce(best, account.clone())
			.map_err(|e| ErrorObject::owned(1, "Unable to query nonce.", Some(e.to_string())))?;
		Ok(get_missing_nonces(&*self.pool, account, nonce))
	}
}

/// Finds any missing nonce values inside Future pool and return them as result
fn get_missing_nonces<P, AccountId, Nonce>(pool: &P, account: AccountId, nonce: Nonce) -> Vec<Nonce>
where
	P: TransactionPool,
	AccountId: Clone + Encode,
	Nonce: Clone + Encode + Decode + AtLeast32Bit + 'static,
{
	// Now we need to query the transaction pool
	// and find transactions originating from the same sender.
	// Since extrinsics are opaque to us, we look for them using
	// `provides` tag. And increment the nonce if we find a transaction
	// that matches the current one.
	let mut current_nonce = nonce.clone();
	let encoded_account = account.clone().encode();
	let mut current_tag = (account.clone(), nonce).encode();
	for tx in pool.ready() {
		// since transactions in `ready()` need to be ordered by nonce
		// it's fine to continue with current iterator.
		if tx.provides().get(0) == Some(&current_tag) {
			current_nonce += One::one();
			current_tag = (account.clone(), current_nonce.clone()).encode();
		}
	}

	let mut result = vec![];
	let mut my_in_future: Vec<_> = pool
		.futures()
		.into_iter()
		.filter_map(|x| match x.provides().get(0) {
			// filtering transactions by account
			Some(tag) if tag.starts_with(&encoded_account) => {
				if let Ok(nonce) = Nonce::decode(&mut &tag[encoded_account.len()..]) {
					return Some(nonce)
				}
				None
			},
			_ => None,
		})
		.collect();
	my_in_future.sort();

	for future_nonce in my_in_future {
		while current_nonce < future_nonce {
			result.push(current_nonce.clone());
			current_nonce += One::one();

			// short circuit if we reached the limit
			if result.len() == MAX_RETURNED_MISSING_NONCE_SIZE {
				return result
			}
		}

		// progress the current_nonce
		current_nonce += One::one();
	}

	result
}
