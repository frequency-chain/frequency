use crate::{homoglyphs::canonical::HandleConverter, tests::mock::*};
use std::{
	fs::File,
	io::{BufRead, BufReader},
};

#[test]
fn canonical_test() {
	let input = "A Α А Ꭺ ᗅ ᴀ ꓮ Ａ 𐊠 𝐀 𝐴 𝑨 𝒜 𝓐 𝔄 𝔸 𝕬 𝖠 𝗔 𝘈 𝘼 𝙰 𝚨 𝛢 𝜜 𝝖 𝞐";
	let normalized = unicode_security::skeleton(input).collect::<String>();
	let result = normalized.to_lowercase(); // convert to lower case
	println!("{}", result);
}

#[test]
fn test_remove_confusables() {
	let file = File::open("src/homoglyphs/confusable_characters.txt");
	assert!(file.is_ok());

	let reader = BufReader::new(file.ok().unwrap());
	let handle_converter = HandleConverter::new();
	for line_result in reader.lines() {
		let original_line: &str = &line_result.ok().unwrap();
		let normalized_line = handle_converter.remove_confusables(original_line);

		println!("{}", normalized_line);
	}
}
