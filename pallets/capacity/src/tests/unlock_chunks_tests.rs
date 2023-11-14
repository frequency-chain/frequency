use crate::{
	tests::mock::{new_test_ext, Test},
	Config, Error, UnlockChunks,
};
use frame_support::{assert_err, assert_ok, traits::Get};

#[test]
fn unlock_chunks_add_returns_error_when_too_many_chunks() {
	new_test_ext().execute_with(|| {
		let mut unlock_chunks: UnlockChunks<Test> = UnlockChunks::default();
		let limit: u32 = <Test as Config>::MaxUnlockingChunks::get();

		for i in 0..limit {
			assert_ok!(unlock_chunks.add(i as u64, i));
		}
		assert_err!(unlock_chunks.add(4, 4), Error::<Test>::MaxUnlockingChunksExceeded);
	})
}

#[test]
fn unlock_chunks_reap_thawed_happy_path() {
	new_test_ext().execute_with(|| {
		let mut unlock_chunks: UnlockChunks<Test> = UnlockChunks::default();
		// 10 token total, 6 token unstaked
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		assert_eq!(true, unlock_chunks.set_unlock_chunks(&new_unlocks));
		assert_eq!(3, unlock_chunks.unlocking.len());

		// At epoch 3, the first two chunks should be thawed.
		assert_eq!(3u64, unlock_chunks.reap_thawed(3u32));
		assert_eq!(1, unlock_chunks.unlocking.len());

		// At epoch 5, all unstaking is done.
		assert_eq!(3u64, unlock_chunks.reap_thawed(5u32));
		assert_eq!(0, unlock_chunks.unlocking.len());

		assert_eq!(0u64, unlock_chunks.reap_thawed(5u32));
	})
}

#[test]
fn unlock_chunks_total() {
	new_test_ext().execute_with(|| {
		let mut unlock_chunks: UnlockChunks<Test> = UnlockChunks::default();
		assert_eq!(0u64, unlock_chunks.total());
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		assert_eq!(true, unlock_chunks.set_unlock_chunks(&new_unlocks));
		assert_eq!(6u64, unlock_chunks.total());
	})
}
