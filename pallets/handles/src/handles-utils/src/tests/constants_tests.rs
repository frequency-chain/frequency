#[path = "../../constants.rs"]
mod constants;
use constants::*;

// Use this to regenerate the compacted ALLOWED_UNICODE_CHARACTER_RANGES.
// You can comment out the current one and uncomment the original, specific one
// for all the languages supported.
#[test]
fn test_build_allowed_char_ranges() {
	let res = build_allowed_char_ranges();
	assert_eq!(res.len(), 54usize);
	for range in res {
		let start = range.start();
		let end = range.end();
		println!("{start:#4X}..={end:#4X},")
	}
}
