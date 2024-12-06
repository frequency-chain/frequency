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

/// testnet specific msa ids for migration
pub fn get_testnet_msa_ids() -> Vec<MessageSourceId> {
	vec![
		8004, 8009, 8816, 8817, 8818, 8819, 8820, 8822, 8823, 8824, 8825, 8826, 9384, 9753, 9919,
		9992, 9994, 9996, 9997, 10009, 10010, 10012, 10013, 10014, 10015, 10019, 10020, 10021,
		10022, 10023, 10024, 10025, 10026, 10027, 10028, 10029, 10030, 10031, 10032, 10033, 10034,
		10035, 10036, 10037, 10038, 10039, 10040, 10041, 10042, 10043, 10044, 10045, 10046, 10047,
		10048, 10049, 10050, 10051, 10052, 10053, 10054, 10055, 10056, 10057, 10058, 10059, 10061,
		10062, 10064, 10067, 10068, 10069, 10070, 10071, 10072, 10075, 10076, 10077, 10078, 10079,
		10138, 10139, 10140, 10206, 10207, 10209, 10212, 10218, 10219, 10220, 10221, 10222, 10223,
		10224, 10231, 10232, 10233, 10234, 10235, 10236, 10237, 10238, 10239, 10240, 10241, 10242,
		10243, 10247, 10248, 10251, 10253, 10254, 10255, 10256, 10257, 10258, 10259, 10260, 10261,
		10262, 10263, 10264, 10265, 10266, 10267, 10268, 10269, 10270, 10271, 10272, 10273, 10274,
		10275, 10287, 10288, 10289, 10290, 10291, 10292, 10293, 10294, 10295, 10296, 10297, 10298,
		10299, 10300, 10301, 10302, 10303, 10304, 10305, 10306, 10307, 10308, 10309, 10311, 10312,
		10313, 10314, 10315, 10316, 10317, 10318, 10319, 10320, 10321, 10322, 10323, 10324, 10325,
		10326, 10327, 10328, 10329,
	]
}

/// returns the chain type from genesis hash
pub fn get_chain_type<T: Config>() -> DetectedChainType {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
	get_chain_type_by_genesis_hash(&genesis.encode()[..])
}

/// get the msa ids with key migrations
pub fn get_msa_ids<T: Config>() -> Vec<MessageSourceId> {
	let chain_type = get_chain_type::<T>();
	if let DetectedChainType::FrequencyMainNet = chain_type {
		vec![
			227, 542, 1249820, 1287729, 1288925, 1309067, 1309241, 1309258, 1309367, 1309397,
			1329112, 1329535, 1330067,
		]
	} else {
		if cfg!(test) {
			// this allows to test the mainnet path
			vec![1]
		} else {
			// we are going to use hooks for this multi-block migration so this is empty to only flag that
			// it as done for consistency
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
	if onchain_version < 1 {
		let msa_ids = get_msa_ids::<T>();
		let weights = migrate_msa_ids::<T>(&msa_ids[..]);
		// Set storage version to `1`.
		StorageVersion::new(1).put::<Pallet<T>>();
		let total_weight = T::DbWeight::get().writes(1).saturating_add(weights);
		log::info!(target: LOG_TARGET, "Migration Calculated weights={:?}",total_weight);
		total_weight
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

/// migrating all msa_ids
pub fn migrate_msa_ids<T: Config>(msa_ids: &[MessageSourceId]) -> Weight {
	let schema_id = get_schema_id();
	let key: ItemizedKey = (schema_id,);
	let each_layer_access: u64 = 33 * 16;
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
	log::info!(target: LOG_TARGET, "Storage migrated to version 1  read={:?}, write={:?}, bytes={:?}", reads, writes, bytes);
	let weights = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
	log::info!(target: LOG_TARGET, "migrate_msa_ids weights={:?}",weights);
	weights
}

/// paginated migration for testnet
pub fn paginated_migration_testnet<T: Config>(page_size: u32, page_index: u32) -> (Weight, bool) {
	let msa_ids: Vec<MessageSourceId> = get_testnet_msa_ids();
	let mut chunks = msa_ids.chunks(page_size as usize);
	let chunk_len = chunks.len() as u32;
	let mut current = 0u32;
	while current < page_index && current < chunk_len {
		let _ = chunks.next();
		current += 1;
	}
	match chunks.next() {
		Some(page) => {
			let weight = migrate_msa_ids::<T>(page);
			(weight, true)
		},
		None => (Weight::zero(), false),
	}
}

fn get_schema_id() -> u16 {
	if cfg!(test) {
		// Supported ITEMIZED_APPEND_ONLY_SCHEMA for tests
		103
	} else {
		7
	}
}
