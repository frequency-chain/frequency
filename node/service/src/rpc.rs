//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use common_primitives::node::{AccountId, Balance, Block, Hash, Index as Nonce};

use sc_client_api::{AuxStore, StorageProvider};
use sc_client_db::Backend as DbBackend;
use sc_consensus_manual_seal::rpc::{EngineCommand, ManualSeal, ManualSealApiServer};
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<EngineCommand<Hash>>>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
	    + StorageProvider<Block, DbBackend<Block>>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_messages_runtime_api::MessagesRuntimeApi<Block>,
	C::Api: pallet_schemas_runtime_api::SchemasRuntimeApi<Block>,
	C::Api: pallet_msa_runtime_api::MsaRuntimeApi<Block, AccountId>,
	P: TransactionPool + Sync + Send + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	// Frequency RPCs
	use pallet_messages_rpc::{MessagesApiServer, MessagesHandler};
	use pallet_msa_rpc::{MsaApiServer, MsaHandler};
	use pallet_schemas_rpc::{SchemasApiServer, SchemasHandler};
	use crate::frequency_rpc::{FrequencyRpcApiServer, FrequencyRpcHandler};

	use sp_runtime::OpaqueExtrinsic;
	use common_primitives::node::BlakeTwo256;
	use sp_core::H256;
	use frequency_runtime::RuntimeEvent;

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe, command_sink } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(MessagesHandler::new(client.clone()).into_rpc())?;
	module.merge(SchemasHandler::new(client.clone()).into_rpc())?;
	module.merge(MsaHandler::new(client.clone()).into_rpc())?;
	module.merge(<FrequencyRpcHandler<sp_runtime::generic::Block<sp_runtime::generic::Header<u32, BlakeTwo256>, OpaqueExtrinsic>, C, sc_client_db::Backend<sp_runtime::generic::Block<sp_runtime::generic::Header<u32, BlakeTwo256>, OpaqueExtrinsic>>> as FrequencyRpcApiServer<H256, RuntimeEvent, Hash>>::into_rpc(FrequencyRpcHandler::new(client)))?;
	if let Some(command_sink) = command_sink {
		module.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}
	Ok(module)
}
