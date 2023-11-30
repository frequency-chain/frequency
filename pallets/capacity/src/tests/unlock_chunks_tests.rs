use crate::{
	tests::mock::{new_test_ext, Test},
	unlock_chunks_from_vec, unlock_chunks_reap_thawed, unlock_chunks_total, Config, Error,
	UnlockChunkList,
};
use sp_runtime::BoundedVec;
use std::ops::Bound;

#[test]
fn unlock_chunks_reap_thawed_happy_path() {
	new_test_ext().execute_with(|| {
		// 10 token total, 6 token unstaked
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		let mut chunks = unlock_chunks_from_vec::<Test>(&new_unlocks);
		assert_eq!(3, chunks.len());

		// At epoch 3, the first two chunks should be thawed.
		let reaped = unlock_chunks_reap_thawed::<Test>(&mut chunks, 3);
		assert_eq!(3, reaped);
		assert_eq!(1, chunks.len());
		assert_eq!(3, unlock_chunks_total::<Test>(&chunks));

		// At epoch 5, all unstaking is done.
		assert_eq!(3u64, unlock_chunks_reap_thawed::<Test>(&mut chunks, 5u32));
		assert_eq!(0, chunks.len());

		assert_eq!(0u64, unlock_chunks_reap_thawed::<Test>(&mut chunks, 5u32));
	})
}

#[test]
fn unlock_chunks_total_works() {
	new_test_ext().execute_with(|| {
		let mut chunks: UnlockChunkList<Test> = BoundedVec::default();
		assert_eq!(0u64, unlock_chunks_total::<Test>(&chunks));
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		chunks = unlock_chunks_from_vec::<Test>(&new_unlocks);
		assert_eq!(6u64, unlock_chunks_total::<Test>(&chunks));
	})
}
