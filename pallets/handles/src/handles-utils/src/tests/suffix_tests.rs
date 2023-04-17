use crate::suffix::{generate_seed, generate_unique_suffixes};

#[test]
fn should_always_have_the_same_seed() {
	assert_eq!(generate_seed("abcdefg"), 15079896798642598352u64);
	assert_eq!(generate_seed("gfedcba"), 9497970330616778036u64);
}


#[test]
fn should_generate_the_same_sequence() {
	assert_eq!(generate_unique_suffixes(10, 99, "abcdefg").take(10).collect::<Vec<u16>>(), vec![49, 28, 22, 87, 57, 39, 69, 54, 79, 37]);
	assert_eq!(generate_unique_suffixes(10, 99, "gfedcba").take(10).collect::<Vec<u16>>(), vec![78, 52, 69, 46, 89, 34, 88, 49, 97, 77]);
}