//! # Schemas Pallet
//!
//! The Schemas pallet provides functionality for handling schemas.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! This pallet provides an on chain repository for schemas, thereby allowing participants of the
//! network to flexibly interact and exchange messages with each other without facing the challenge
//! of sharing, managing and validating messages as well as schemas between them.
//!
//! <b>NOTE</b>: In this pallet we define the <b>payload</b> structure that is used in <a href="../pallet_messages/index.html">Messages Pallet</a>.
//!
//! The Schema pallet provides functions for:
//!
//! - Registering a new schema.
//! - Setting maximum schema model size by governance.
//! - Retrieving latest registered schema id.
//! - Retrieving schemas by their id.
//!
//!
//! ### Terminology
//!
//! - **Schema:** The structure that defines how a Message is stored and structured.
//!
//! - **Schema Model:** Serialization/Deserialization details of the schema
//!
//! - **Schema Model Type:** The type of the following Serialization/Deserialization. It can be
//!   Avro, Parquet or ...
//!
//! ### Dispatchable Functions
//!
//! - `register_schema` - Registers a new schema after some initial validation.
//! - `set_max_schema_model_bytes` - Sets the maximum schema model size (Bytes) by governance.
//!
//! The Schema pallet implements the following traits:
//!
//! - [`SchemaProvider`](common_primitives::schema::SchemaProvider<SchemaId>): Functions for accessing and validating Schemas.  This implementation is what is used in the runtime.
//!
//! ## Genesis config
//!
//! The Schemas pallet depends on the [`GenesisConfig`].
//!
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

mod storage;
mod types;

use codec::Encode;
use frame_support::{ensure, traits::Get, BoundedVec};
use sp_runtime::DispatchError;
use sp_std::prelude::*;

pub use pallet::*;
pub use storage::*;
pub use types::*;
pub use weights::*;

use common_primitives::msa::MessageSourceId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::msa::MessageSourceId;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// weight info
		type WeightInfo: WeightInfo;

		/// The maximum amount of follows a single account can have.
		#[pallet::constant]
		type MaxNodes: Get<u32>;

		/// max follows for a node
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

	/// mapping between static_id -> node
	#[pallet::storage]
	#[pallet::getter(fn get_node)]
	pub(super) type Nodes<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, Node, OptionQuery>;

	/// static_id -> [edge, edge, ...]
	#[pallet::storage]
	#[pallet::getter(fn graph_adj)]
	pub(super) type GraphAdj<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, BoundedVec<Edge, T::MaxFollows>, ValueQuery>;

	/// double storage
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn graph_map)]
	pub(super) type GraphMap<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MessageSourceId,
		Twox64Concat,
		MessageSourceId,
		Permission,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// dummy
		ActionNotPermitted,
		/// dummy
		SigningFailed,
		/// dummy
		SignatureInvalid,
		/// dummy
		NoSuchNode,
		/// dummy
		NoSuchEdge,
		/// dummy
		NoSuchPage,
		/// dummy
		NoSuchStaticId,
		/// dummy
		NodeExists,
		/// dummy
		EdgeExists,
		/// dummy
		EdgeExistsInPublicGraph,
		/// dummy
		InvalidEdge,
		/// dummy
		TooManyNodes,
		/// dummy
		TooManyEdges,
		/// dummy
		InvalidSecret,
		/// dummy
		SelfFollowNotPermitted,
	}

	/// events of graph
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a new Static Id is registered. [who, staticId]
		NodeAdded(T::AccountId, MessageSourceId),

		/// Event emitted when a follow has been added. [who, staticId, staticId]
		Followed(T::AccountId, MessageSourceId, MessageSourceId),

		/// Event emitted when a follow has been removed. [who, staticId, staticId]
		Unfollowed(T::AccountId, MessageSourceId, MessageSourceId),

		/// Event emitted when a follow has been modified. [who, staticId]
		Modified(T::AccountId, MessageSourceId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dummy docs for add_node
		#[pallet::weight(T::WeightInfo::add_node(*static_id as u32) )]
		pub fn add_node(origin: OriginFor<T>, static_id: MessageSourceId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Nodes::<T>::try_mutate(static_id, |maybe_node| -> DispatchResult {
				ensure!(maybe_node.is_none(), Error::<T>::NodeExists);
				let cur_count = Self::node_count();
				let node_id = cur_count.checked_add(1).ok_or(<Error<T>>::TooManyNodes)?;

				*maybe_node = Some(Node {});
				<NodeCount<T>>::set(node_id);
				Self::deposit_event(Event::NodeAdded(sender, static_id));
				log::debug!("Node added: {:?} -> {:?}", static_id, node_id);
				Ok(())
			})?;

			Ok(())
		}

		/// follow docs
		#[pallet::weight(T::WeightInfo::follow_adj(*from_static_id as u32))]
		pub fn follow_adj(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let edge = Edge { static_id: to_static_id, permission: Permission { data: 0 } };

			<GraphAdj<T>>::try_mutate(&from_static_id, |edge_vec| {
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

		/// unfollow docs
		#[pallet::weight(T::WeightInfo::unfollow_adj(*from_static_id as u32))]
		pub fn unfollow_adj(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let cur_count: u64 = Self::edge_count();
			ensure!(cur_count > 0, <Error<T>>::NoSuchEdge);

			<GraphAdj<T>>::try_mutate(&from_static_id, |edge_vec| {
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

		/// follow_2 docs
		#[pallet::weight(T::WeightInfo::follow_map(*from_static_id as u32))]
		pub fn follow_map(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let perm = <GraphMap<T>>::try_get(from_static_id, to_static_id);
			ensure!(perm.is_err(), <Error<T>>::EdgeExists);

			<GraphMap<T>>::insert(from_static_id, to_static_id, Permission { data: 0 });

			let cur_count: u64 = Self::edge_count();
			<EdgeCount<T>>::set(cur_count + 1);

			Self::deposit_event(Event::Followed(sender, from_static_id, to_static_id));

			log::debug!("followed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		/// unfollow2 docs
		#[pallet::weight(T::WeightInfo::unfollow_map(*from_static_id as u32))]
		pub fn unfollow_map(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			let perm = <GraphMap<T>>::try_get(from_static_id, to_static_id);
			ensure!(perm.is_ok(), <Error<T>>::NoSuchEdge);

			let cur_count: u64 = Self::edge_count();
			ensure!(cur_count > 0, <Error<T>>::NoSuchEdge);

			<GraphMap<T>>::remove(from_static_id, to_static_id);

			<EdgeCount<T>>::set(cur_count - 1);
			Self::deposit_event(Event::Unfollowed(sender, from_static_id, to_static_id));

			log::debug!("unfollowed: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		/// child graph public follow
		#[pallet::weight(T::WeightInfo::follow_child_public(*from_static_id as u32))]
		pub fn follow_child_public(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
			permission: Permission,
			page: u16,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self follow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			ensure!(<Nodes<T>>::contains_key(to_static_id), <Error<T>>::NoSuchNode);

			let key = Self::get_storage_key(&permission, page);
			let perm = Storage::<T>::read_public_graph(&from_static_id, &key.clone());

			let mut edges: Vec<MessageSourceId> = Vec::new();
			match perm {
				Some(p) => {
					edges = p.into_inner();
					match edges.binary_search(&to_static_id) {
						Ok(_) => Err(<Error<T>>::EdgeExists),
						Err(index) => {
							edges.insert(index, to_static_id);
							Ok(())
						},
					}?;
				},
				None => edges.push(to_static_id),
			}

			let p = PublicPage::try_from(edges).map_err(|_| <Error<T>>::TooManyEdges)?;
			Storage::<T>::write_public(&from_static_id, &key, Some(p.into()))?;

			Self::deposit_event(Event::Followed(sender, from_static_id, to_static_id));

			log::debug!("followed child: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		/// child graph public unfollow
		#[pallet::weight(T::WeightInfo::unfollow_child_public(*from_static_id as u32))]
		pub fn unfollow_child_public(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			to_static_id: MessageSourceId,
			permission: Permission,
			page: u16,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// self unfollow is not permitted
			ensure!(from_static_id != to_static_id, <Error<T>>::SelfFollowNotPermitted);
			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);

			let key = Self::get_storage_key(&permission, page);
			let perm = Storage::<T>::read_public_graph(&from_static_id, &key.clone());
			ensure!(perm.is_some(), <Error<T>>::NoSuchEdge);

			let mut edge_vec = perm.unwrap();
			match edge_vec.binary_search(&to_static_id) {
				Ok(index) => {
					edge_vec.remove(index);
					Ok(())
				},
				Err(_) => Err(<Error<T>>::NoSuchEdge),
			}?;

			// removing an item should not need to check bounded size
			let p = PublicPage::try_from(edge_vec).unwrap();
			Storage::<T>::write_public(
				&from_static_id,
				&key,
				if p.len() > 0 { Some(p) } else { None },
			)?;

			Self::deposit_event(Event::Unfollowed(sender, from_static_id, to_static_id));

			log::debug!("unfollowed child: {:?} -> {:?}", from_static_id, to_static_id);
			Ok(())
		}

		/// private graph update
		#[pallet::weight((0, Pays::No))]
		pub fn private_graph_update(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			permission: Permission,
			page: u16,
			value: PrivatePage,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			let key = Self::get_storage_key(&permission, page);

			Storage::<T>::write_private(
				&from_static_id,
				&key,
				if value.len() > 0 { Some(value) } else { None },
			)?;

			Self::deposit_event(Event::Modified(sender, from_static_id));

			log::debug!("modified: {:?}", from_static_id);
			Ok(())
		}

		/// change page number, used to swap last page number with the page that just got removed
		#[pallet::weight((0, Pays::No))]
		pub fn change_page_number(
			origin: OriginFor<T>,
			from_static_id: MessageSourceId,
			graph_type: GraphType,
			permission: Permission,
			from_page: u16,
			to_page: u16,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(<Nodes<T>>::contains_key(from_static_id), <Error<T>>::NoSuchNode);
			let from_key = Self::get_storage_key(&permission, from_page);
			let to_key = Self::get_storage_key(&permission, to_page);

			match graph_type {
				GraphType::Public => {
					let from = Storage::<T>::read_public_graph(&from_static_id, &from_key.clone());
					ensure!(from.is_some(), <Error<T>>::NoSuchPage);
					let to = Storage::<T>::read_public_graph(&from_static_id, &to_key.clone());
					ensure!(to.is_none(), <Error<T>>::NoSuchPage);
					Storage::<T>::write_public(&from_static_id, &to_key, from)?;
					Storage::<T>::write_public(&from_static_id, &from_key, None)?;
				},
				GraphType::Private => {
					let from = Storage::<T>::read_private_graph(&from_static_id, &from_key.clone());
					ensure!(from.is_some(), <Error<T>>::NoSuchPage);
					let to = Storage::<T>::read_private_graph(&from_static_id, &to_key.clone());
					ensure!(to.is_none(), <Error<T>>::NoSuchPage);
					Storage::<T>::write_private(&from_static_id, &to_key, from)?;
					Storage::<T>::write_private(&from_static_id, &from_key, None)?;
				},
			};

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// get neighbors from adj graph
	pub fn get_following_list_public(
		static_id: MessageSourceId,
	) -> Result<Vec<MessageSourceId>, DispatchError> {
		ensure!(<Nodes<T>>::contains_key(static_id), <Error<T>>::NoSuchNode);
		let graph = <GraphAdj<T>>::get(static_id);

		Ok(graph.into_iter().map(|e| e.static_id).collect())
	}

	/// get storage key
	pub fn get_storage_key(permission: &Permission, page: u16) -> StorageKey {
		let key = GraphKey { permission: *permission, page };
		let mut buf: Vec<u8> = Vec::new();
		buf.extend_from_slice(&key.encode()[..]);
		StorageKey::try_from(buf).unwrap()
	}

	/// read from child tree
	pub fn read_public_graph_node(
		static_id: MessageSourceId,
		key: StorageKey,
	) -> Option<PublicPage> {
		if let Some(_) = Self::get_node(static_id) {
			return Storage::<T>::read_public_graph(&static_id, &key)
		}
		None
	}

	/// read all public keys
	pub fn read_public_graph(static_id: MessageSourceId) -> Vec<(GraphKey, PublicPage)> {
		let mut v = Vec::new();
		if let Some(_) = Self::get_node(static_id) {
			let tree_nodes = Storage::<T>::public_graph_iter(&static_id);
			for n in tree_nodes {
				v.push(n);
			}
		}
		v
	}

	/// read from child tree
	pub fn read_private_graph_node(
		static_id: MessageSourceId,
		key: StorageKey,
	) -> Option<PrivatePage> {
		if let Some(_) = Self::get_node(static_id) {
			return Storage::<T>::read_private_graph(&static_id, &key)
		}
		None
	}

	/// read all private keys
	pub fn read_private_graph(static_id: MessageSourceId) -> Vec<(GraphKey, PrivatePage)> {
		let mut v = Vec::new();
		if let Some(_) = Self::get_node(static_id) {
			let tree_nodes = Storage::<T>::private_graph_iter(&static_id);
			for n in tree_nodes {
				v.push(n);
			}
		}
		v
	}
}
