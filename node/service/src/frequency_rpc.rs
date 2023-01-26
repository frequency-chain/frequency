use std::fmt::Debug;
use std::sync::Arc;
use codec::{Codec, EncodeLike, Decode};
use frame_benchmarking::frame_support::pallet_prelude::TypeInfo;
use jsonrpsee::{
	proc_macros::rpc,
	core::{Error as RpcError, RpcResult},
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sc_client_api::{Backend, StorageProvider};
use sp_runtime::{traits::Block as BlockT};
use sp_core::storage::{StorageData, StorageKey};
use frame_system::EventRecord;
use serde::{Deserialize, Deserializer};

/// Frequency MSA Custom RPC API
#[rpc(client, server)]
pub trait FrequencyRpcApi<BlockHash, RuntimeEvent, Hash>
where
	Hash: Decode + Sync + Send + TypeInfo,
	RuntimeEvent: Decode + Sync + Send + TypeInfo + Debug + Eq + Clone + EncodeLike + 'static
{
	/// gets the events for a block hash
	#[method(name = "frequency_getEvents")]
	fn get_events(
		&self,
		block_hash: BlockHash
	) -> RpcResult<Option<Vec<EventRecord<RuntimeEvent, Hash>>>>;
}


/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct FrequencyRpcHandler<Block, Client, BE>
{
	client: Arc<Client>,
	_marker: std::marker::PhantomData<(Block, BE)>,
}

impl<Block, Client, BE> FrequencyRpcHandler<Block, Client, BE> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<Client>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<Block, Client, BE, RuntimeEvent, Hash> FrequencyRpcApiServer<<Block as BlockT>::Hash, RuntimeEvent, Hash> for FrequencyRpcHandler<Block, Client, BE>
	where
		Block: BlockT + 'static,
		Block::Hash: Unpin,
		BE: Backend<Block> + 'static,
		Client: StorageProvider<Block, BE> + Send + Sync + 'static,
		Hash: Decode + Sync + Send + TypeInfo,
		RuntimeEvent: Decode + Sync + Send + TypeInfo + Debug + Eq + Clone + EncodeLike + 'static
{
	fn get_events(
		&self,
		block_hash: <Block as BlockT>::Hash
	) -> RpcResult<Option<Vec<EventRecord<RuntimeEvent, Hash>>>> {
		let decoded = hex::decode("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").expect("Decoding failed");
		log::info!("inside get_events with {:?}", decoded);
		let storage =  self.client.storage(block_hash, &StorageKey(decoded))
			.map_err( |e| {
				log::info!("error {:?}", e);
				RpcError::Call(CallError::Custom(ErrorObject::owned(
				ErrorCode::ServerError(300).code(),
			"Unable to get state",
			Some(format!("{:?}", e)),
				)))
			})?;
		if let Some(data) = storage {
			return Ok(<Vec<EventRecord<RuntimeEvent, Hash>>>::codec::Decode(&mut &data.0[..]).unwrap())
		}
		Ok(None)
	}
}

// impl Deserialize for EventRecord<E, T> {
// 	fn deserialize<D>(deserializer: D) -> Result<Self, serde::de::Error> where D: Deserializer<'de> {
// 		todo!()
// 	}
//
// 	fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), serde::de::Error> where D: Deserializer<'de> {
// 		todo!()
// 	}
// }
