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

mod frequency_rpc;

#[cfg(test)]
mod tests;

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
	C::Api: pallet_frequency_tx_payment_rpc::CapacityTransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_messages_runtime_api::MessagesRuntimeApi<Block>,
	C::Api: pallet_schemas_runtime_api::SchemasRuntimeApi<Block>,
	C::Api: system_runtime_api::AdditionalRuntimeApi<Block>,
	C::Api: pallet_msa_runtime_api::MsaRuntimeApi<Block, AccountId>,
	C::Api: pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi<Block>,
	C::Api: pallet_handles_runtime_api::HandlesRuntimeApi<Block>,
	P: TransactionPool + Sync + Send + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	// Frequency RPCs
	use frequency_rpc::{FrequencyRpcApiServer, FrequencyRpcHandler};
	use pallet_frequency_tx_payment_rpc::{CapacityPaymentApiServer, CapacityPaymentHandler};
	use pallet_handles_rpc::{HandlesApiServer, HandlesHandler};
	use pallet_messages_rpc::{MessagesApiServer, MessagesHandler};
	use pallet_msa_rpc::{MsaApiServer, MsaHandler};
	use pallet_schemas_rpc::{SchemasApiServer, SchemasHandler};
	use pallet_stateful_storage_rpc::{StatefulStorageApiServer, StatefulStorageHandler};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe, command_sink } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(MessagesHandler::new(client.clone()).into_rpc())?;
	module.merge(SchemasHandler::new(client.clone()).into_rpc())?;
	module.merge(MsaHandler::new(client.clone()).into_rpc())?;
	module.merge(StatefulStorageHandler::new(client.clone()).into_rpc())?;
	module.merge(HandlesHandler::new(client.clone()).into_rpc())?;
	module.merge(CapacityPaymentHandler::new(client.clone()).into_rpc())?;
	module.merge(FrequencyRpcHandler::new(client).into_rpc())?;
	if let Some(command_sink) = command_sink {
		module.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}
	Ok(module)
}
