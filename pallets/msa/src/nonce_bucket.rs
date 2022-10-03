use crate::{Config, Error};
use common_primitives::node::{BlockNumber, Signature};
use frame_system::Config as SystemConfig;
use sp_runtime::{bounded_vec, traits::ConstU32, BoundedVec};
use std::intrinsics::{floorf32, floorf64};

pub const MORTALITY_SIZE: u64 = 200;
pub const SIGNATURES_MAX: u32 = 10_000;

pub struct NonceBucket<T: SystemConfig> {
	pub mortality_block: T::BlockNumber,
	pub signatures: BoundedVec<Signature, ConstU32<SIGNATURES_MAX>>,
}

impl<T: SystemConfig> NonceBucket<T> {
	pub fn new(mortality_block: T::BlockNumber) -> Self {
		Self { mortality_block, signatures: bounded_vec![] }
	}
}

type BucketNumber = u8;
pub const NUM_BUCKETS: BucketNumber = 2;
pub struct NonceBuckets<T: SystemConfig> {
	buckets: Vec<NonceBucket<T>>,
}

impl<T> NonceBuckets<T> {
	pub fn push_signature(
		sig: Signature,
		current_block: BlockNumber,
	) -> Result<BucketNumber, Error<T>> {
		Ok(0)
	}
	pub fn valid_signature(sig: Signature, current_block: BlockNumber) -> bool {
		false
	}
	pub fn bucket_for(current_block: BlockNumber) -> BucketNumber {
		(current_block / 100) % 2;
	}
}
