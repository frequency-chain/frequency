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
fn test_convert_confuseables_to_unicode_escaped() {
	let file = File::open("src/tests/confuseable_characters.txt");
	assert!(file.is_ok());

	let reader = BufReader::new(file.ok().unwrap());

	println!("use sp_std::collections::btree_map::BTreeMap;");
	println!("");
	println!("const CHAR_MAP: BTreeMap<char, char> = BTreeMap::from([");

	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();
		let mut original_line_characters = original_line.chars();

		// The first character in the line will be the value of each key
		let normalized_character = original_line_characters.next().unwrap();

		while let Some(homoglyph) = original_line_characters.next() {
			print!("(\'\\u{{{:x}}}\',", homoglyph as u32);
			println!(" \'\\u{{{:x}}}\'),", normalized_character as u32);
		}
	}

	println!("]);");
}
