use sp_runtime::MultiSignature;
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

use common_primitives::node::{Hash, Signature};

use crate::nonce_bucket::NonceBucketsError::NoSuchBucket;

/// this is a block number :P
pub type BlockNumber = u32;

/// NonceBucket stores signatures that expire at BlockNumber `mortality_block`
#[derive(Clone, Debug, PartialEq)]
pub struct NonceBucket {
	/// when this bucket can be cleared
	pub mortality_block: BlockNumber,
	/// Map that stores the hashes of signatures as keys to an individual block number
	/// which is the ______ ?
	pub signature_hashes: BTreeMap<Hash, u32>,
}

impl NonceBucket {
	/// new - returns an instance of NonceBucket with the mortality block set to `mortality_block`
	pub fn new(mortality_block: BlockNumber) -> Self {
		Self { mortality_block, signature_hashes: BTreeMap::new() }
	}
}

/// Mortality bucket size
pub const MORTALITY_SIZE: u32 = 100;
/// Number of buckets to sort signatures into
const NUM_BUCKETS: u8 = 2;
/// Limit on number of signatures to store in a bucket
pub const SIGNATURES_MAX: u32 = 50_000;

/// NonceBuckets stores a Vec<NonceBucket> of size NUM_BUCKETS
/// Used for checking signatures against their mortality block.
/// mortality blocks are rotated to save storage.
#[derive(Clone)]
pub struct NonceBuckets {
	/// a list of NonceBuckets for storing signatures with their expiration (mortality) block
	buckets: Vec<NonceBucket>,
}

/// Get the bucket for a given block
pub fn bucket_for(block_number: u32) -> u8 {
	(block_number / MORTALITY_SIZE % (NUM_BUCKETS as u32)) as u8
}

/// hash_multisignature takes a MultiSignature and hashes the signature by getting
/// it as_ref.
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

/// NonceBucketsError - Error enum for errors when interacting with NonceBuckets.
#[derive(Debug, PartialEq)]
pub enum NonceBucketsError {
	/// Attempted to add a signature when the signature is already in the registry
	BucketKeyExists,
	/// Attempted to add a signature with a mortality too far in the future
	MortalityTooHigh,
	/// Attempted to retrieve a bucket outside the bounds of storage
	NoSuchBucket,
}

/// NonceBuckets instance functions
impl NonceBuckets {
	// static fns

	/// new returns in an instance of NonceBuckets with the provided starting current block
	/// A bucket number is determined by the current block.
	/// The mortality block is when all the signatures in the bucket will expire.
	pub fn new(current_block: BlockNumber) -> Self {
		let mut bucket_mortality_block = (current_block / MORTALITY_SIZE + 1) * MORTALITY_SIZE;
		let mut buckets: Vec<NonceBucket> =
			vec![NonceBucket::new(bucket_mortality_block); NUM_BUCKETS as usize];

		// cycle through the buckets and set the mortality block
		// note: bucket_num is not always going to be zero, because it depends on the value
		// of current_block. So we simply iterate as many times as there are buckets.
		let mut bucket_num: usize = bucket_for(current_block) as usize;
		for _i in 0..NUM_BUCKETS {
			let bucket = buckets.get_mut(bucket_num).unwrap();
			bucket.mortality_block = bucket_mortality_block;
			bucket_mortality_block += MORTALITY_SIZE;
			if bucket_num == (NUM_BUCKETS - 1) as usize {
				bucket_num = 0;
			} else {
				bucket_num += 1;
			}
		}
		Self { buckets }
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
		mortality_block: BlockNumber,
	) -> Result<BlockNumber, NonceBucketsError> {
		let key: Hash = hash_multisignature(sig);

		if current_block + MORTALITY_SIZE * (NUM_BUCKETS as u32) < mortality_block {
			Err(NonceBucketsError::MortalityTooHigh)
		} else {
			let bucket_num = bucket_for(mortality_block);
			if let Some(bucket) = self.buckets.get_mut(bucket_num as usize) {
				// check to see if it's time to clear this bucket based on the current block.
				if current_block > bucket.mortality_block {
					bucket.signature_hashes.clear();
				}
				match bucket.signature_hashes.insert(key, mortality_block) {
					Some(_) => Err(NonceBucketsError::BucketKeyExists),
					None => Ok(bucket_for(mortality_block) as u32),
				}
			} else {
				// This shouldn't happen but it's required to make the compiler happy.
				Err(NoSuchBucket)
			}
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
	pub fn instantiates_a_bucket_correctly() {
		new_test_ext().execute_with(|| {
			let bucket = NonceBucket::new(5);
			assert_eq!(bucket.mortality_block, 5);
		})
	}

	#[test]
	pub fn bucket_for_works() {
		new_test_ext().execute_with(|| {
			for i in [0, 1, 2, 3, 99, 222, 77_400] {
				assert_eq!(0, bucket_for(i));
			}
			for j in [333, 1101, 100, 111, 99_900, 66_700] {
				assert_eq!(1, bucket_for(j));
			}
		})
	}

	#[test]
	pub fn instantiates_buckets_correctly() {
		new_test_ext().execute_with(|| {
			let buckets = NonceBuckets::new(50_099);

			// The "even" bucket signatures should expire at the hundred block
			let test_bucket1 = buckets.get_bucket(0).unwrap();
			assert_eq!(50_100, test_bucket1.mortality_block);

			// The "odd" bucket signatures should expire at the two-hundred block
			let test_bucket2 = buckets.get_bucket(1).unwrap();
			assert_eq!(50_200, test_bucket2.mortality_block);

			let result = buckets.get_bucket(2);
			assert_err!(result, NonceBucketsError::NoSuchBucket);

			// should be bucket 1
			let buckets2 = NonceBuckets::new(3_333_333);
			let test_bucket = buckets2.get_bucket(1).unwrap();
			assert_eq!(3_333_400, test_bucket.mortality_block);

			let test_bucket = buckets2.get_bucket(0).unwrap();
			assert_eq!(3_333_500, test_bucket.mortality_block);
		})
	}

	#[test]
	pub fn puts_signature_in_correct_bucket() {
		new_test_ext().execute_with(|| {
			let current_block: BlockNumber = 11_233;
			let mortality_block: BlockNumber = 11_333;

			let buckets = &mut NonceBuckets::new(current_block);

			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone(), mortality_block));

			// signature goes in "odd" bucket
			let test_bucket = buckets.get_bucket(1).unwrap();
			assert_eq!(false, test_bucket.signature_hashes.is_empty());

			// expect signature to expire at 400
			assert_eq!(11_400, test_bucket.mortality_block);

			let sighash = hash_multisignature(&(sig1.clone()));

			let res = test_bucket.signature_hashes.get(&sighash);
			assert_ne!(None, res);
			assert_eq!(Some(&mortality_block), res);
		});
	}

	#[test]
	pub fn cannot_add_signature_twice() {
		new_test_ext().execute_with(|| {
			let current_block: BlockNumber = 11_122;
			let mortality_block: BlockNumber = 11_222;
			let buckets = &mut NonceBuckets::new(current_block);

			let sig1 = &generate_test_signature();
			assert_ok!(buckets.push_signature(sig1, current_block.clone(), mortality_block));

			let res = buckets.push_signature(sig1, current_block.clone(), mortality_block.clone());
			assert_eq!(Err(BucketKeyExists), res);
		})
	}

	#[test]
	pub fn clears_first_bucket_at_correct_block() {
		// Test that we put signatures in first bucket when current block > start_block + (mortality size*num buckets)
		// Test that we clear the bucket before adding new signatures.
		new_test_ext().execute_with(|| {
			let mut current_block: BlockNumber = 11_122;
			let mut mortality_block: BlockNumber = 11_299;
			let mut buckets = NonceBuckets::new(current_block);

			let sig1 = &generate_test_signature();
			let sig2 = &generate_test_signature();
			let sig3 = &generate_test_signature();

			{
				assert_ok!(buckets.push_signature(sig1, current_block.clone(), mortality_block));
				let first_bucket = buckets.get_bucket(0).unwrap();
				assert_eq!(1, first_bucket.signature_hashes.len());
			}

			current_block += 100;
			mortality_block += 100;
			{
				assert_ok!(buckets.push_signature(sig2, current_block.clone(), mortality_block));
				let last_bucket = buckets.get_bucket(1).unwrap();
				assert_eq!(1, last_bucket.signature_hashes.len());
			}

			current_block += 100;
			mortality_block += 100;
			{
				assert_ok!(buckets.push_signature(sig3, current_block.clone(), mortality_block));
				// ensure there is only one signature and it's not the old one
				let first_bucket = buckets.get_bucket(0).unwrap();
				assert_eq!(1, first_bucket.signature_hashes.len());

				let sighash1 = hash_multisignature(&(sig1.clone()));
				let sighash3 = hash_multisignature(&(sig3.clone()));

				let expected_none = first_bucket.signature_hashes.get(&sighash1);
				assert_eq!(None, expected_none);
				let expected_some = first_bucket.signature_hashes.get(&sighash3);
				assert_eq!(Some(&mortality_block), expected_some);
			}
		})
	}
}
