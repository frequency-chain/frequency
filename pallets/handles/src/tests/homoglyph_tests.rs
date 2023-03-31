use crate::homoglyphs::canonical::HandleConverter;

use std::{
	fs::File,
	io::{BufRead, BufReader},
};

#[test]
fn test_replace_confusables() {
	let file = File::open("src/homoglyphs/confusable_characters.txt");
	assert!(file.is_ok());

	let reader = BufReader::new(file.ok().unwrap());
	let handle_converter = HandleConverter::new();
	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();

		// The first character in `confusable_characters.txt` is the normalized character
		// that each subsequent character may be confused with
		let first_character = original_line.chars().next().unwrap();

		let normalized_line = handle_converter.replace_confusables(&original_line);
		for normalized_character in normalized_line.chars() {
			let normalized_character_codepoint =
				format!("\'\\u{{{:x}}}\'", normalized_character as u32);
			let first_character_codepoint = format!("\'\\u{{{:x}}}\'", first_character as u32);
			// println!("normalized_character_codepoint: {}  first_character_codepoint: {}", normalized_character_codepoint, first_character_codepoint);

			assert_eq!(first_character_codepoint, normalized_character_codepoint);
		}
	}
}

#[test]
fn test_strip_diacriticals() {
	let handle_converter = HandleConverter::new();
	let diacritical_string = "ÄÅÖäåöĂăĔĚĕĞğģĬĭŎŏŬǓŭàáâñ";
	let stripped_string = handle_converter.strip_diacriticals(diacritical_string);
	assert_eq!(stripped_string, "AAOaaoAaEEeGggIiOoUUuaaan");
}
