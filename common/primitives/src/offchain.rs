use crate::msa::MessageSourceId;
use numtoa::NumToA;
use parity_scale_codec::Decode;
#[cfg(feature = "std")]
use sp_externalities::ExternalitiesExt;
use sp_runtime::offchain::storage::{StorageRetrievalError, StorageValueRef};
use sp_runtime_interface::runtime_interface;
use sp_std::{fmt::Debug, vec, vec::Vec};

#[cfg(feature = "std")]
sp_externalities::decl_extension! {
	/// Offchain worker custom extension
	pub struct OcwCustomExt (
		// rpc address provided to offchain worker
		Vec<u8>
	);
}

/// host functions for custom extension
#[cfg(feature = "std")]
pub type CustomExtensionHostFunctions = (custom::HostFunctions,);

/// runtime new customized
#[runtime_interface]
pub trait Custom: ExternalitiesExt {
	/// another function
	fn get_val(&mut self) -> Option<Vec<u8>> {
		self.extension::<OcwCustomExt>().map(|ext| ext.0.clone())
	}
}
/// Lock expiration timeout in in milli-seconds for msa pallet per msa account
pub const MSA_ACCOUNT_LOCK_TIMEOUT_EXPIRATION_MS: u64 = 50;
/// Lock name prefix for msa account
pub const MSA_ACCOUNT_LOCK_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::lock::";
/// Offchain storage prefix for msa account
pub const MSA_ACCOUNT_STORAGE_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::keys::";
/// msa account lock name
pub fn get_msa_account_lock_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_LOCK_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}
/// msa account storage key name
pub fn get_msa_account_storage_key_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_STORAGE_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}

/// Locks the execution of the function
#[derive(Debug)]
pub enum LockStatus {
	/// Lock is acquired
	Locked,
	/// Lock is released
	Released,
}

/// Wrapper for offchain get operations
pub fn get_index_value<V: Decode + Debug>(key: &[u8]) -> Result<Option<V>, StorageRetrievalError> {
	get_impl::<V>(key)
}

/// Gets a value by the key from persistent storage
fn get_impl<V: Decode + Debug>(key: &[u8]) -> Result<Option<V>, StorageRetrievalError> {
	let oci_mem = StorageValueRef::persistent(key);
	match oci_mem.get::<V>() {
		Ok(Some(data)) => Ok(Some(data)),
		Ok(None) => Ok(None),
		Err(_) => Err(StorageRetrievalError::Undecodable),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::offchain::{testing, OffchainDbExt, OffchainWorkerExt};
	use sp_io::TestExternalities;

	#[test]
	fn get_msa_account_lock_name_should_return_expected_value() {
		let msa_id: MessageSourceId = 2_000_000;
		let result = get_msa_account_lock_name(msa_id);
		assert_eq!(result, b"Msa::ofw::lock::2000000".to_vec());
	}

	#[test]
	fn get_msa_account_storage_name_should_return_expected_value() {
		let msa_id: MessageSourceId = 2_000_000;
		let result = get_msa_account_storage_key_name(msa_id);
		assert_eq!(result, b"Msa::ofw::keys::2000000".to_vec());
	}

	#[test]
	fn get_index_for_not_set_should_return_none() {
		let (offchain, _state) = testing::TestOffchainExt::new();
		let mut t = TestExternalities::default();
		t.register_extension(OffchainDbExt::new(offchain.clone()));
		t.register_extension(OffchainWorkerExt::new(offchain));

		t.execute_with(|| {
			let key = b"my_key";
			let result = get_index_value::<MessageSourceId>(key);
			assert_eq!(result, Ok(None));
		});
	}

	#[test]
	fn get_index_for_set_should_return_expected() {
		// arrange
		let (offchain, _state) = testing::TestOffchainExt::new();
		let mut t = TestExternalities::default();
		t.register_extension(OffchainDbExt::new(offchain.clone()));
		t.register_extension(OffchainWorkerExt::new(offchain));

		t.execute_with(|| {
			let key = b"my_key1";
			let msa_id: MessageSourceId = 1000000;
			let oci_mem = StorageValueRef::persistent(key);
			oci_mem.set(&msa_id);

			// act
			let result = get_index_value::<MessageSourceId>(key);

			// assert
			assert_eq!(result, Ok(Some(msa_id)));
		});
	}

	#[test]
	fn get_index_for_not_decodable_should_return_error() {
		let (offchain, _state) = testing::TestOffchainExt::new();
		let mut t = TestExternalities::default();
		t.register_extension(OffchainDbExt::new(offchain.clone()));
		t.register_extension(OffchainWorkerExt::new(offchain));

		#[derive(Debug, Decode, PartialEq)]
		struct Testing {
			pub a: u64,
			pub b: u32,
			pub c: u16,
		}

		t.execute_with(|| {
			// arrange
			let key = b"my_key2";
			let msa_id: MessageSourceId = 1000000;
			let oci_mem = StorageValueRef::persistent(key);
			oci_mem.set(&msa_id);

			// act
			let result = get_index_value::<Testing>(key);

			// assert
			assert_eq!(result, Err(StorageRetrievalError::Undecodable));
		});
	}
}
