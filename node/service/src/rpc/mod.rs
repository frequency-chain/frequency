//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use common_primitives::node::{AccountId, Balance, Block, Hash, Index as Nonce};

use cumulus_primitives_core::BlockT;
use sc_client_api::{backend::Backend, AuxStore, StorageProvider};
use sc_client_db::Backend as DbBackend;
use sc_consensus_manual_seal::rpc::{EngineCommand, ManualSeal, ManualSealApiServer};
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_inherents::CreateInherentDataProviders;

mod frequency_rpc;

mod eth;
pub use self::eth::{create_eth, EthDeps};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<B: BlockT, C, P, CT, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<EngineCommand<Hash>>>,
	/// Ethereum-compatibility specific dependencies.
	pub eth: EthDeps<B, C, P, CT, CIDP>,
}

pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<B, C, BE> fc_rpc::EthConfig<B, C> for DefaultEthConfig<C, BE>
where
	B: BlockT,
	C: StorageProvider<B, BE> + Sync + Send + 'static,
	BE: Backend<B> + 'static,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride =
		fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<B, C, BE>;
}

/// Instantiate all RPC extensions.
pub fn create_full<B, C, P, BE, CT, CIDP, OffchainDB>(
	deps: FullDeps<B, C, P, CT, CIDP>,
	offchain: Option<OffchainDB>,
	subscription_task_executor: SubscriptionTaskExecutor,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<B>,
		>,
	>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	B: BlockT + for<'de> serde::Deserialize<'de>,
	C: CallApiAt<B>
		+ ProvideRuntimeApi<B>
		+ HeaderBackend<B>
		+ AuxStore
		+ HeaderMetadata<B, Error = BlockChainError>
		+ StorageProvider<B, DbBackend<B>>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<B, Balance>,
	C::Api: pallet_frequency_tx_payment_rpc::CapacityTransactionPaymentRuntimeApi<B, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<B, AccountId, Nonce>,
	C::Api: BlockBuilder<B>,
	C::Api: pallet_messages_runtime_api::MessagesRuntimeApi<B>,
	C::Api: pallet_schemas_runtime_api::SchemasRuntimeApi<B>,
	C::Api: system_runtime_api::AdditionalRuntimeApi<B>,
	C::Api: pallet_msa_runtime_api::MsaRuntimeApi<B, AccountId>,
	C::Api: pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi<B>,
	C::Api: pallet_handles_runtime_api::HandlesRuntimeApi<B>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<B>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
	OffchainDB: sp_core::offchain::OffchainStorage + 'static,
	P: TransactionPool + Sync + Send + 'static,
	BE: Backend<B> + 'static,
	CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
	CT: fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
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
	let FullDeps { client, pool, command_sink, eth } = deps;

	module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(MessagesHandler::new(client.clone()).into_rpc())?;
	module.merge(SchemasHandler::new(client.clone()).into_rpc())?;
	module.merge(MsaHandler::new(client.clone(), offchain).into_rpc())?;
	module.merge(StatefulStorageHandler::new(client.clone()).into_rpc())?;
	module.merge(HandlesHandler::new(client.clone()).into_rpc())?;
	module.merge(CapacityPaymentHandler::new(client.clone()).into_rpc())?;
	module.merge(FrequencyRpcHandler::new(client, pool).into_rpc())?;
	if let Some(command_sink) = command_sink {
		module.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}

	// Ethereum compatibility RPCs
	let io = create_eth::<_, _, _, _, _, _, DefaultEthConfig<C, BE>>(
		module,
		eth,
		subscription_task_executor,
		pubsub_notification_sinks,
	)?;

	Ok(io)
}
