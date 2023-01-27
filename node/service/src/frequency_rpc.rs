use codec::{Decode, Encode};
use frame_system::{EventRecord, Phase};
use jsonrpsee::{
	core::{Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sc_client_api::{Backend, StorageProvider};
use sp_core::{
	storage::{StorageData, StorageKey},
	H256,
};
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

/// Frequency MSA Custom RPC API
#[rpc(client, server)]
pub trait FrequencyRpcApi<BlockHash> {
	/// gets the events for a block hash
	#[method(name = "frequency_getEvents")]
	fn get_events(
		&self,
		block_hash: BlockHash,
		extrinsic_indices: Vec<u32>,
	) -> RpcResult<Option<StorageData>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct FrequencyRpcHandler<Block, Client, BE> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<(Block, BE)>,
}

impl<Block, Client, BE> FrequencyRpcHandler<Block, Client, BE> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<Client>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<Block, Client, BE> FrequencyRpcApiServer<<Block as BlockT>::Hash>
	for FrequencyRpcHandler<Block, Client, BE>
where
	Block: BlockT + 'static,
	Block::Hash: Unpin,
	BE: Backend<Block> + 'static,
	Client: StorageProvider<Block, BE> + Send + Sync + 'static,
{
	fn get_events(
		&self,
		block_hash: <Block as BlockT>::Hash,
		extrinsic_indices: Vec<u32>,
	) -> RpcResult<Option<StorageData>> {
		let decoded =
			hex::decode("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
				.expect("Decoding failed");
		log::info!("inside get_events with {:?}", decoded);
		let storage = self.client.storage(block_hash, &StorageKey(decoded)).map_err(|e| {
			log::info!("error {:?}", e);
			RpcError::Call(CallError::Custom(ErrorObject::owned(
				ErrorCode::ServerError(300).code(),
				"Unable to get state",
				Some(format!("{:?}", e)),
			)))
		})?;
		if let Some(data) = storage {
			let events: Vec<EventRecord<frequency_runtime::RuntimeEvent, H256>> =
				Decode::decode(&mut &data.0[..]).unwrap();
			let filtered: Vec<_> = events
				.into_iter()
				.filter(|e| match e.phase {
					Phase::ApplyExtrinsic(i) if extrinsic_indices.contains(&i) => true,
					_ => false,
				})
				.collect();
			log::info!("{:?}", filtered);
			return Ok(Some(StorageData(Encode::encode(&filtered))))
		}
		Ok(None)
	}
}
