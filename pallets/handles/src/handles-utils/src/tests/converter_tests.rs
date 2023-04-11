use crate::converter::HandleConverter;

// use std::{
// 	fs::File,
// 	io::{BufRead, BufReader},
// };

// #[test]
// fn test_replace_confusables() {
// 	let file = File::open("src/utils/confusable_characters.txt");
// 	assert!(file.is_ok());

// 	let reader = BufReader::new(file.ok().unwrap());
// 	for line_result in reader.lines() {
// 		let original_line = line_result.ok().unwrap();

// 		// The first character in `confusable_characters.txt` is the normalized character
// 		// that each subsequent character may be confused with
// 		let first_character = original_line.chars().next().unwrap();

// 		let normalized_line = HandleConverter::replace_confusables(&original_line);
// 		for normalized_character in normalized_line.chars() {
// 			let normalized_character_codepoint =
// 				format!("\'\\u{{{:x}}}\'", normalized_character as u32);
// 			let first_character_codepoint = format!("\'\\u{{{:x}}}\'", first_character as u32);
// 			// println!("normalized_character_codepoint: {}  first_character_codepoint: {}", normalized_character_codepoint, first_character_codepoint);

// 			assert_eq!(first_character_codepoint, normalized_character_codepoint);
// 		}
// 	}
// }

#[test]
fn test_strip_diacriticals() {
	let diacritical_string = "ÄÅÖäåöĂăĔĚĕĞğģĬĭŎŏŬǓŭàáâñ";
	let stripped_string = HandleConverter::strip_diacriticals(diacritical_string);
	assert_eq!(stripped_string, "AAOaaoAaEEeGggIiOoUUuaaan");
}

#[test]
fn test_strip_unicode_whitespace() {
	let whitespace_chars = vec![
		'\u{0009}', // tab
		'\u{000A}', // line feed
		'\u{000B}', // vertical tab
		'\u{000C}', // form feed
		'\u{000D}', // carriage return
		'\u{0020}', // space
		'\u{0085}', // next line
		'\u{00A0}', // no-break space
		'\u{1680}', // ogham space mark
		'\u{2000}', // en quad
		'\u{2001}', // em quad
		'\u{2002}', // en space
		'\u{2003}', // em space
		'\u{2004}', // three-per-em space
		'\u{2005}', // four-per-em space
		'\u{2006}', // six-per-em space
		'\u{2007}', // figure space
		'\u{2008}', // punctuation space
		'\u{2009}', // thin space
		'\u{200A}', // hair space
		'\u{2028}', // line separator
		'\u{2029}', // paragraph separator
		'\u{202F}', // narrow no-break space
		'\u{205F}', // medium mathematical space
		'\u{3000}', // ideographic space
	];
	let whitespace_string: String = whitespace_chars.into_iter().collect();
	let string_with_whitespace =
		format!("{}hello{}world!{}", whitespace_string, whitespace_string, whitespace_string);
	println!("String with whitespace: {}", string_with_whitespace);
	let whitespace_stripped_string =
		HandleConverter::strip_unicode_whitespace(&string_with_whitespace);
	println!("Whitespace stripped string: {}", whitespace_stripped_string);
	assert_eq!(whitespace_stripped_string, "helloworld!");
}
