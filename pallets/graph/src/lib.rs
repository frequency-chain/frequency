// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

mod storage;
mod types;

use codec::{Decode, Encode};
use frame_support::{ensure, traits::Get, BoundedVec, Hashable, StorageHasher, Twox64Concat};
use sp_runtime::DispatchError;
use sp_std::prelude::*;

pub use pallet::*;
pub use types::*;
pub use weights::*;

use crate::storage::Storage;
use common_primitives::msa::MessageSenderId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::storage::Storage;
	use common_primitives::msa::MessageSenderId;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// The maximum amount of follows a single account can have.
		#[pallet::constant]
		type MaxNodes: Get<u32>;

		#[pallet::constant]
		type MaxFollows: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Keeps track of the number of nodes in existence.
	#[pallet::storage]
	#[pallet::getter(fn node_count)]
	pub(super) type NodeCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Keeps track of the number of edges in existence.
	#[pallet::storage]
	#[pallet::getter(fn edge_count)]
	pub(super) type EdgeCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	// mapping between static_id -> node
	#[pallet::storage]
	#[pallet::getter(fn get_node)]
	pub(super) type Nodes<T: Config> =
		StorageMap<_, Twox64Concat, MessageSenderId, Node, OptionQuery>;

	// static_id -> [edge, edge, ...]
	#[pallet::storage]
	#[pallet::getter(fn graph)]
	pub(super) type Graph<T: Config> =
		StorageMap<_, Twox64Concat, MessageSenderId, BoundedVec<Edge, T::MaxFollows>, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn graph2)]
	pub(super) type Graph2<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MessageSenderId,
		Twox64Concat,
		MessageSenderId,
		Permission,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		ActionNotPermitted,
		SigningFailed,
		SignatureInvalid,
		NoSuchNode,
		NoSuchEdge,
		NoSuchStaticId,
		NodeExists,
		EdgeExists,
		EdgeExistsInPublicGraph,
		InvalidEdge,
		TooManyNodes,
		TooManyEdges,
		InvalidSecret,
		SelfFollowNotPermitted,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a new Static Id is registered. [who, staticId]
		NodeAdded(T::AccountId, MessageSenderId),

		/// Event emitted when a follow has been added. [who, staticId, staticId]
		Followed(T::AccountId, MessageSenderId, MessageSenderId),

		/// Event emitted when a follow has been removed. [who, staticId, staticId]
		Unfollowed(T::AccountId, MessageSenderId, MessageSenderId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::add_node(*static_id as u32) )]
		pub fn add_node(origin: OriginFor<T>, static_id: MessageSenderId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Nodes::<T>::try_mutate(static_id, |maybe_node| -> DispatchResult {
				ensure!(maybe_node.is_none(), Error::<T>::NodeExists);
				let cur_count = Self::node_count();
				let node_id = cur_count.checked_add(1).ok_or(<Error<T>>::TooManyNodes)?;

				*maybe_node = Some(Node { trie_id: Storage::<T>::generate_trie_id(static_id, 1) });
				<NodeCount<T>>::set(node_id);
				Self::deposit_event(Event::NodeAdded(sender, static_id));
				log::debug!("Node added: {:?} -> {:?}", static_id, node_id);
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::follow(*from_static_id as u32))]
		pub fn follow(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let edge = Edge { static_id: to_static_id, permission: Permission { data: 0 } };

			<Graph<T>>::try_mutate(&from_static_id, |edge_vec| {
				match edge_vec.binary_search(&edge) {
					Ok(_) => Err(<Error<T>>::EdgeExists),
					Err(index) =>
						edge_vec.try_insert(index, edge).map_err(|_| <Error<T>>::TooManyEdges),
				}
			})?;

			let cur_count: u64 = Self::edge_count();
			<EdgeCount<T>>::set(cur_count + 1);

			Self::deposit_event(Event::Followed(sender, from_static_id, to_static_id));

			log::debug!("followed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unfollow(*from_static_id as u32))]
		pub fn unfollow(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let cur_count: u64 = Self::edge_count();
			ensure!(cur_count > 0, <Error<T>>::NoSuchEdge);

			<Graph<T>>::try_mutate(&from_static_id, |edge_vec| {
				let edge = Edge { static_id: to_static_id, permission: Permission { data: 0 } };
				match edge_vec.binary_search(&edge) {
					Ok(index) => {
						edge_vec.remove(index);
						Ok(())
					},
					Err(_) => Err(()),
				}
			})
			.map_err(|_| <Error<T>>::NoSuchEdge)?;

			<EdgeCount<T>>::set(cur_count - 1);
			Self::deposit_event(Event::Unfollowed(sender, from_static_id, to_static_id));

			log::debug!("unfollowed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::follow(*from_static_id as u32))]
		pub fn follow2(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let perm = <Graph2<T>>::try_get(from_static_id, to_static_id);
			ensure!(perm.is_err(), <Error<T>>::EdgeExists);

			<Graph2<T>>::insert(from_static_id, to_static_id, Permission { data: 0 });

			let cur_count: u64 = Self::edge_count();
			<EdgeCount<T>>::set(cur_count + 1);

			Self::deposit_event(Event::Followed(sender, from_static_id, to_static_id));

			log::debug!("followed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unfollow(*from_static_id as u32))]
		pub fn unfollow2(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			let perm = <Graph2<T>>::try_get(from_static_id, to_static_id);
			ensure!(perm.is_ok(), <Error<T>>::NoSuchEdge);

			let cur_count: u64 = Self::edge_count();
			ensure!(cur_count > 0, <Error<T>>::NoSuchEdge);

			<Graph2<T>>::remove(from_static_id, to_static_id);

			<EdgeCount<T>>::set(cur_count - 1);
			Self::deposit_event(Event::Unfollowed(sender, from_static_id, to_static_id));

			log::debug!("unfollowed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::follow(*from_static_id as u32))]
		pub fn follow3(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			let from_node = Self::get_node(from_static_id);
			ensure!(from_node.is_some(), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let trie_id = from_node.unwrap().trie_id;
			let perm = Storage::<T>::read(&trie_id, &Self::get_storage_key(to_static_id));
			ensure!(perm.is_none(), <Error<T>>::EdgeExists);

			let data = Permission { data: 1 };
			Storage::<T>::write(
				&trie_id,
				&Self::get_storage_key(to_static_id),
				Some(data.encode().to_vec()),
			)?;

			Self::deposit_event(Event::Followed(sender, from_static_id, to_static_id));

			log::debug!("followed 3: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unfollow(*from_static_id as u32))]
		pub fn unfollow3(
			origin: OriginFor<T>,
			from_static_id: MessageSenderId,
			to_static_id: MessageSenderId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			let from_node = Self::get_node(from_static_id);
			ensure!(from_node.is_some(), <Error<T>>::NoSuchNode);

			let trie_id = from_node.unwrap().trie_id;
			let perm = Storage::<T>::read(&trie_id, &Self::get_storage_key(to_static_id));
			ensure!(perm.is_some(), <Error<T>>::NoSuchEdge);

			Storage::<T>::write(&trie_id, &Self::get_storage_key(to_static_id), None)?;

			Self::deposit_event(Event::Unfollowed(sender, from_static_id, to_static_id));

			log::debug!("unfollowed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn get_following_list_public(
		static_id: MessageSenderId,
	) -> Result<Vec<MessageSenderId>, DispatchError> {
		ensure!(<Nodes<T>>::contains_key(static_id), <Error<T>>::NoSuchNode);
		let graph = <Graph<T>>::get(static_id);

		Ok(graph.into_iter().map(|e| e.static_id).collect())
	}

	pub fn get_storage_key(static_id: MessageSenderId) -> StorageKey {
		#[cfg(test)]
		{
			use std::{println as info, println as warn};
			println!("{} to_le_bytes {:X?}", static_id, static_id.encode());
		}
		StorageKey::try_from(static_id.encode()).unwrap()
	}

	pub fn read_from_child_tree(static_id: MessageSenderId, key: StorageKey) -> Option<Vec<u8>> {
		if let Some(node) = Self::get_node(static_id) {
			return Storage::<T>::read(&node.trie_id, &key)
		}
		None
	}

	pub fn read_all_keys(static_id: MessageSenderId) -> Vec<MessageSenderId> {
		if let Some(node) = Self::get_node(static_id) {
			return Storage::<T>::iter_keys(&node.trie_id)
				.iter()
				.map(|v| {
					let mut m = MessageSenderId::default();
					for (i, b) in v.iter().enumerate() {
						m += (*b as MessageSenderId) * (1u64 << (i * 8));
					}
					m.into()
				})
				.collect()
		}
		Vec::new()
	}
}
