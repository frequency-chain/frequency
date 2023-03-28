use std::{
	fs::File,
	io::{BufRead, BufReader},
};

// Helper function to generate a BTreeMap which maps each confusable character to its
// canonical counterpart for each line in `confusable_characters.txt`.
//
// The first character of each line represent the canonical character (value) in the map that each
// subsequent character of the line (key) maps to.
fn convert_confuseables_to_unicode_escaped() {
	let file = File::open("src/tests/homoglyps/confuseable_characters.txt");
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
