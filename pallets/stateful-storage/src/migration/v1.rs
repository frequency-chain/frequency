#[cfg(feature = "try-runtime")]
use crate::types::STATEFUL_STORAGE_VERSION;
use crate::{
	stateful_child_tree::StatefulChildTree,
	types::{
		ItemAction, ItemizedKey, ItemizedOperations, ItemizedPage, Page, ITEMIZED_STORAGE_PREFIX,
		PALLET_STORAGE_PREFIX,
	},
	Config, Pallet, LOG_TARGET,
};
use common_primitives::{
	msa::MessageSourceId,
	utils::{get_chain_type_by_genesis_hash, DetectedChainType},
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
#[cfg(feature = "try-runtime")]
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use sp_std::{vec, vec::Vec};

fn get_schema_id() -> u16 {
	if cfg!(test) {
		103
	} else {
		7
	}
}

/// get the msa ids with key migrations
pub fn get_msa_ids<T: Config>() -> Vec<MessageSourceId> {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);

	if let DetectedChainType::FrequencyMainNet =
		get_chain_type_by_genesis_hash(&genesis.encode()[..])
	{
		vec![
			227, 542, 1249820, 1287729, 1288925, 1309067, 1309241, 1309258, 1309367, 1309397,
			1329112, 1329535, 1330067,
		]
	} else {
		if cfg!(test) {
			vec![1]
		} else {
			vec![]
		}
	}
}

/// migration to v1 implementation
pub struct MigrateToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
	fn on_runtime_upgrade() -> Weight {
		migrate_to_v1::<T>()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		let genesis_block: BlockNumberFor<T> = 0u32.into();
		let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
		if on_chain_version >= 1 {
			return Ok(Vec::new())
		}
		log::info!(target: LOG_TARGET, "Found genesis... {:?}", genesis);
		let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
		log::info!(target: LOG_TARGET,"Detected Chain is {:?}", detected_chain);
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version > 1 {
			return Ok(())
		}
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(onchain_version, STATEFUL_STORAGE_VERSION);
		// check to ensure updates took place
		let schema_id = get_schema_id();
		let msa_ids = get_msa_ids::<T>();
		for msa_id in msa_ids.into_iter() {
			let itemized: ItemizedPage<T> = Pallet::<T>::get_itemized_page_for(msa_id, schema_id)
				.map_err(|_| TryRuntimeError::Other("can not get storage"))?
				.ok_or(TryRuntimeError::Other("no storage"))?;
			let (_, val) = <Page<T::MaxItemizedPageSizeBytes> as ItemizedOperations<T>>::try_parse(
				&itemized, false,
			)
			.map_err(|_| TryRuntimeError::Other("can not parse storage"))?
			.items
			.clone()
			.into_iter()
			.next()
			.ok_or(TryRuntimeError::Other("no item"))?;

			assert_eq!(val.len(), 33);
			log::info!(target: LOG_TARGET, "{:?}", HexDisplay::from(&val));
		}
		log::info!(target: LOG_TARGET, "Finished post_upgrade");
		Ok(())
	}
}

/// migrating to v1
pub fn migrate_to_v1<T: Config>() -> Weight {
	log::info!(target: LOG_TARGET, "Running storage migration...");
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::in_code_storage_version();
	log::info!(target: LOG_TARGET, "onchain_version= {:?}, current_version={:?}", onchain_version, current_version);
	let schema_id = get_schema_id();
	let key: ItemizedKey = (schema_id,);
	let each_layer_access: u64 = 33 * 16;

	if onchain_version < 1 {
		let msa_ids = get_msa_ids::<T>();
		let mut reads = 1u64;
		let mut writes = 0u64;
		let mut bytes = 0u64;

		for msa_id in msa_ids.iter() {
			reads.saturating_inc();
			// get the itemized storages
			let itemized_result: Result<Option<ItemizedPage<T>>, _> =
				Pallet::<T>::get_itemized_page_for(*msa_id, schema_id);
			match itemized_result {
				Ok(Some(existing_page)) => {
					bytes = bytes.saturating_add(existing_page.encode().len() as u64);
					bytes = bytes.saturating_add(each_layer_access * 3); // three layers in merkle tree

					match <Page<T::MaxItemizedPageSizeBytes> as ItemizedOperations<T>>::try_parse(
						&existing_page,
						false,
					) {
						Ok(parsed_page) => match parsed_page.items.clone().into_iter().next() {
							Some((_, existing_value)) => match existing_value.len() {
								32usize => {
									// 64 is decimal value for 0x40
									let mut prefixed = vec![64u8];
									prefixed.extend_from_slice(existing_value);
									let bounded: BoundedVec<u8, T::MaxItemizedBlobSizeBytes> =
										prefixed.try_into().unwrap_or_default();

									let empty_page = ItemizedPage::<T>::default();
									match <Page<<T as Config>::MaxItemizedPageSizeBytes> as ItemizedOperations<T>>::apply_item_actions(&empty_page,&vec![ItemAction::Add {
										data: bounded,
									}]) {
										Ok(mut updated_page) => {
											updated_page.nonce = existing_page.nonce;
											StatefulChildTree::<T::KeyHasher>::write(
												msa_id,
												PALLET_STORAGE_PREFIX,
												ITEMIZED_STORAGE_PREFIX,
												&key,
												&updated_page,
											);
											bytes = bytes.saturating_add(updated_page.encode().len() as u64);
											writes.saturating_inc();
										},
										Err(e) =>
											log::error!(target: LOG_TARGET, "Error appending prefixed value {:?} and schema_id {:?} with {:?}", msa_id, schema_id, e),
									}
								},
								33usize =>
									log::warn!(target: LOG_TARGET, "Itemized page item for msa_id {:?} and schema_id {:?} has correct size", msa_id, schema_id),
								_ =>
									log::warn!(target: LOG_TARGET, "Itemized page item for msa_id {:?} and schema_id {:?} has invalid size {:?}", msa_id, schema_id, existing_value.len()),
							},
							None =>
								log::warn!(target: LOG_TARGET, "Itemized page was empty for msa_id {:?} and schema_id {:?}", msa_id, schema_id),
						},
						Err(e) =>
							log::error!(target: LOG_TARGET, "Error parsing page for msa_id {:?} and schema_id {:?} with {:?}", msa_id, schema_id, e),
					}
				},
				Ok(None) =>
					log::warn!(target: LOG_TARGET, "No page found for msa_id {:?} and schema_id {:?}", msa_id, schema_id),
				Err(e) =>
					log::error!(target: LOG_TARGET, "Error getting the page for msa_id {:?} and schema_id {:?} with {:?}", msa_id, schema_id, e),
			}
		}

		// Set storage version to `1`.
		StorageVersion::new(1).put::<Pallet<T>>();
		writes.saturating_inc();

		log::info!(target: LOG_TARGET, "Storage migrated to version 1  read={:?}, write={:?}, bytes={:?}", reads, writes, bytes);
		let weights = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
		log::info!(target: LOG_TARGET, "Migration Calculated weights={:?}",weights);
		weights
	} else {
		log::info!(
			target: LOG_TARGET,
			"Migration did not execute. This probably should be removed onchain:{:?}, current:{:?}",
			onchain_version,
			current_version
		);
		T::DbWeight::get().reads(1)
	}
}
