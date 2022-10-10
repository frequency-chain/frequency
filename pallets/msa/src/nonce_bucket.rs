// use sp_io::hashing::blake2_256;
use sp_runtime::MultiSignature;
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

use common_primitives::node::{BlockNumber, Hash, Signature};

use crate::nonce_bucket::NonceBucketsError::NoSuchBucket;

/// NonceBucket stores signatures that expire at BlockNumber `mortality_block`
#[derive(Clone, Debug, PartialEq)]
pub struct NonceBucket {
	/// when this bucket is no longer valid
	pub mortality_block: BlockNumber,
	/// signature map that stores the current block number when the signature was provided
	pub signature_hashes: BTreeMap<Hash, u32>,
}

impl NonceBucket {
	/// new - returns an instance of NonceBucket with the mortality block set to `mortality_block`
	pub fn new(mortality_block: BlockNumber) -> Self {
		Self { mortality_block, signature_hashes: BTreeMap::new() }
	}
}

/// Mortality bucket size
pub const MORTALITY_SIZE: BlockNumber = 100;
/// Number of buckets to sort signatures into
const NUM_BUCKETS: BlockNumber = 2;
/// Limit on number of signatures to store in a bucket
pub const SIGNATURES_MAX: u32 = 10_000;

/// NonceBuckets stores a Vec<NonceBucket> of size NUM_BUCKETS
/// Used for checking signatures against their mortality block.
/// mortality blocks are rotated to save storage.
#[derive(Clone)]
pub struct NonceBuckets {
	/// a list of NonceBuckets for storing signatures with their expiration (mortality) block
	buckets: Vec<NonceBucket>,
}

/// NonceBucketsError - Error enum for errors when interacting with NonceBuckets.
#[derive(Debug, PartialEq)]
pub enum NonceBucketsError {
	/// Attempted to add a signature when the signature is already in the registry
	BucketKeyExists,
	/// Attempted to retrieve a bucket outside the bounds of storage
	NoSuchBucket,
}

/// NonceBuckets instance functions
impl NonceBuckets {
	// static fns

	/// Get the bucket for a given block
	pub fn bucket_for(block_number: BlockNumber) -> BlockNumber {
		block_number / MORTALITY_SIZE % NUM_BUCKETS
	}

	/// hash_multisignature takes a MultiSignature and calls Blakc2Hasher::hash by getting
	/// its ref
	pub fn hash_multisignature(signature: &MultiSignature) -> Hash {
		match signature {
			MultiSignature::Ed25519(ref signature) =>
				sp_io::hashing::blake2_256(signature.as_ref()).into(),
			MultiSignature::Sr25519(ref signature) =>
				sp_io::hashing::blake2_256(signature.as_ref()).into(),
			MultiSignature::Ecdsa(ref signature) =>
				sp_io::hashing::blake2_256(signature.as_ref()).into(),
		}
	}

	/// new returns in an instance of NonceBuckets with the provided starting current block
	/// A bucket number is determined by the current block.
	/// The mortality block is when all the signatures in the bucket will expire.
	pub fn new(current_block: BlockNumber) -> Self {
		let mut starting_block = current_block;
		let mut mortality_block = Self::mortality_block_for(current_block);
		let mut buckets: Vec<NonceBucket> =
			vec![NonceBucket::new(mortality_block); NUM_BUCKETS as usize];

		// cycle through the buckets and set the mortality block
		for _i in 0..NUM_BUCKETS {
			let bucket_num: usize = Self::bucket_for(starting_block) as usize;
			let bucket = buckets.get_mut(bucket_num).unwrap();
			bucket.mortality_block = mortality_block;
			mortality_block += MORTALITY_SIZE;
			starting_block += MORTALITY_SIZE;
		}
		Self { buckets }
	}

	/// mortality_block_for gets the expected mortality block for `block_number`.
	/// the mortality block is when a registered signature expires.
	fn mortality_block_for(block_number: BlockNumber) -> u32 {
		// stupid computer math tricks
		(block_number / MORTALITY_SIZE + 1) * MORTALITY_SIZE
	}

	// Instance fns

	/// push_signature adds a new signature and the block it's associated with to storage.
	/// ** Returns **
	/// * Ok(bucket_number) if the signature was pushed
	/// * Error if the signature exists
	pub fn push_signature(
		&mut self,
		sig: &Signature,
		current_block: BlockNumber,
	) -> Result<BlockNumber, NonceBucketsError> {
		let key: Hash = Self::hash_multisignature(sig);

		if self.is_signature_live(sig, current_block) {
			Err(NonceBucketsError::BucketKeyExists)
		} else {
			let bucket_num = Self::bucket_for(current_block);
			if let Some(bucket) = self.buckets.get_mut(bucket_num as usize) {
				// let mut sig_hashes = bucket.signature_hashes;
				if Self::mortality_block_for(current_block) != bucket.mortality_block {
					bucket.signature_hashes.clear();
				}
				match bucket.signature_hashes.insert(key, current_block) {
					// should not happen, for completeness because `try_insert` is experimental
					Some(_) => Err(NonceBucketsError::BucketKeyExists),
					None => Ok(Self::bucket_for(current_block)),
				}
			} else {
				Err(NoSuchBucket)
			}
		}
	}

	/// checks that the signature exists and is not expired
	/// Returns: true if the signature is registered and not expired
	/// Returns false if the signature is expired or isn't registered.
	pub fn is_signature_live(&self, sig: &Signature, current_block: BlockNumber) -> bool {
		let bucket = self.get_bucket(Self::bucket_for(current_block)).unwrap();
		let sighash = Self::hash_multisignature(sig);
		match bucket.signature_hashes.get_key_value(&sighash) {
			None => false,
			Some(_) => current_block <= bucket.mortality_block,
		}
	}

	/// get_bucket gets a bucket index at `bucket_num` for read-only
	/// ** Returns ** a Result<&NonceBucket, NonceBucketsErr>
	pub fn get_bucket(&self, bucket_num: BlockNumber) -> Result<&NonceBucket, NonceBucketsError> {
		match self.buckets.get(bucket_num as usize) {
			None => Err(NonceBucketsError::NoSuchBucket),
			Some(b) => Ok(b),
		}
	}
}

#[cfg(test)]
mod test_nonce_buckets {
	use frame_support::{assert_err, assert_ok};
	use sp_core::{sr25519, Pair};
	use sp_runtime::{testing::H256, MultiSignature};

	use crate::{mock::new_test_ext, nonce_bucket::NonceBucketsError::BucketKeyExists};

	use super::*;

	fn generate_test_signature() -> MultiSignature {
		let (key_pair, _) = sr25519::Pair::generate();
		let fake_data = H256::random();
		key_pair.sign(fake_data.as_bytes()).into()
	}

	#[test]
	pub fn instantiate_a_bucket_correctly() {
		new_test_ext().execute_with(|| {
			let bucket = NonceBucket::new(5);
			assert_eq!(bucket.mortality_block, 5);
		})
	}

	#[test]
	pub fn bucket_for_works() {
		new_test_ext().execute_with(|| {
			for i in [1, 2, 3, 49, 222, 22200] {
				assert_eq!(0, NonceBuckets::bucket_for(i));
			}
			for j in [333, 1101, 111, 99900, 33300] {
				assert_eq!(1, NonceBuckets::bucket_for(j));
			}
		})
	}

	#[test]
	pub fn instantiates_buckets_correctly() {
		new_test_ext().execute_with(|| {
			let buckets = NonceBuckets::new(50_099);

			// The "even" bucket signatures should expire at the hundred block
			let mut test_bucket = buckets.get_bucket(0).unwrap();
			assert_eq!(50_100, test_bucket.mortality_block);

			// The "odd" bucket signatures should expire at the two-hundred block
			test_bucket = buckets.get_bucket(1).unwrap();
			assert_eq!(test_bucket.mortality_block, 50_200);

			let result = buckets.get_bucket(2);
			assert_err!(result, NonceBucketsError::NoSuchBucket);

			// should be bucket 1
			let buckets2 = NonceBuckets::new(3_333_333);
			test_bucket = buckets2.get_bucket(1).unwrap();
			assert_eq!(3_333_400, test_bucket.mortality_block);

			test_bucket = buckets2.get_bucket(0).unwrap();
			assert_eq!(3_333_500, test_bucket.mortality_block);
		})
	}

	#[test]
	pub fn puts_signature_in_correct_bucket() {
		new_test_ext().execute_with(|| {
			let mut current_block: BlockNumber = 11_333;
			let buckets = &mut NonceBuckets::new(current_block);
			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone()));

			// signature goes in "odd" bucket
			let test_bucket = buckets.get_bucket(1).unwrap();
			assert_eq!(false, test_bucket.signature_hashes.is_empty());

			// expect signature to expire at 400
			assert_eq!(11_400, test_bucket.mortality_block);

			let sighash = NonceBuckets::hash_multisignature(&(sig1.clone()));

			let res = test_bucket.signature_hashes.get(&sighash);
			assert_ne!(None, res);
			assert_eq!(Some(&current_block), res);
		});
	}

	#[test]
	pub fn cannot_add_signature_twice() {
		new_test_ext().execute_with(|| {
			let current_block: BlockNumber = 11_222;
			let buckets = &mut NonceBuckets::new(current_block);
			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone()));
			let mut res = buckets.push_signature(sig1, current_block.clone());
			assert_eq!(Err(BucketKeyExists), res);
		})
	}

	#[test]
	pub fn clears_first_bucket_at_correct_block() {
		// Test that we put signatures in first bucket when current block > start_block + (mortality size*num buckets)
		// Test that we clear the bucket before adding new signatures.
		new_test_ext().execute_with(|| {
			let mut current_block: BlockNumber = 11_222;
			let mut buckets = NonceBuckets::new(current_block);

			let sig1 = &generate_test_signature();
			let sig2 = &generate_test_signature();
			let sig3 = &generate_test_signature();

			{
				assert_ok!(buckets.push_signature(sig1, current_block.clone()));
				let first_bucket = buckets.get_bucket(0).unwrap();
				assert_eq!(1, first_bucket.signature_hashes.len());
			}

			current_block += 100;
			{
				assert_ok!(buckets.push_signature(sig2, current_block.clone()));
				let last_bucket = buckets.get_bucket(1).unwrap();
				assert_eq!(1, last_bucket.signature_hashes.len());
			}

			current_block += 100;
			{
				assert_ok!(buckets.push_signature(sig3, current_block.clone()));
				// ensure there is only one signature and it's not the old one
				let first_bucket = buckets.get_bucket(0).unwrap();
				assert_eq!(1, first_bucket.signature_hashes.len());

				let sighash1 = NonceBuckets::hash_multisignature(&(sig1.clone()));
				let sighash3 = NonceBuckets::hash_multisignature(&(sig3.clone()));

				let expected_none = first_bucket.signature_hashes.get(&sighash1);
				assert_eq!(None, expected_none);
				let expected_some = first_bucket.signature_hashes.get(&sighash3);
				assert_eq!(Some(&current_block), expected_some);
			}
		})
	}

	#[test]
	pub fn signature_is_valid_when_found_in_correct_bucket_and_before_mortality() {
		new_test_ext().execute_with(|| {
			let mut current_block: BlockNumber = 3_838_433;
			let buckets = &mut NonceBuckets::new(current_block);
			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone()));

			// signature goes in "even" bucket
			let test_bucket = buckets.get_bucket(0).unwrap();
			assert_eq!(false, test_bucket.signature_hashes.is_empty());
			assert_eq!(true, buckets.is_signature_live(sig1, current_block));
			current_block += 10;
			assert_eq!(true, buckets.is_signature_live(sig1, current_block));
		});
	}

	#[test]
	pub fn signature_is_invalid_when_not_found_in_correct_bucket() {
		new_test_ext().execute_with(|| {
			let mut current_block: BlockNumber = 3_838_433;
			let buckets = &mut NonceBuckets::new(current_block);
			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone()));

			// signature goes in "even" bucket
			let test_bucket = buckets.get_bucket(0).unwrap();
			current_block += MORTALITY_SIZE;
			assert_eq!(false, buckets.is_signature_live(sig1, current_block));

			let sig2 = &generate_test_signature();
			assert_eq!(false, buckets.is_signature_live(sig2, current_block));
		});
	}
}
