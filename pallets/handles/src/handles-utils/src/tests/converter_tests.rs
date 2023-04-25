use crate::{
	convert_to_canonical,
	converter::{
		replace_confusables, split_display_name, strip_diacriticals, strip_unicode_whitespace,
	},
	validator::{
		consists_of_supported_unicode_character_sets, contains_blocked_characters, is_valid,
	},
};

use std::{
	fs::File,
	io::{BufRead, BufReader},
};

#[test]
fn wil_fuzz() {
	let file = File::open("src/data/fuzz.txt");
	assert!(file.is_ok());

	let reader = BufReader::new(file.ok().unwrap());
	let mut valid = 0;
	let mut rejected = 0;
	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();
		let canonical = convert_to_canonical(&original_line);
		if !crate::is_reserved_handle(original_line.as_str()) && is_valid(&canonical) {
			valid += 1;
			// println!("Original: {}\nCanonical: {}\n", original_line, canonical);
			// for c in canonical.chars() {
			// 	println!("Valid char: \"{}\" Unicode: \"{}\"\n\n", c, c.escape_unicode());
			// }
		} else {
			rejected += 1;
			if !contains_blocked_characters(&canonical) {
				println!("REJECTED Original: {}\nCanonical: {}\n", original_line, canonical);
				for c in canonical.chars() {
					let s = c.to_string();
					if !is_valid(s.as_str()) {
						// println!("Unable to validate char: \"{}\" Unicode: \"{}\"", c, c.escape_unicode());
						// println!("Is Valid Range Char?: \"{}\"", consists_of_supported_unicode_character_sets(s.as_str()));
					}
				}
			}
		}
	}
	println!("Successful: {}\nRejected: {}\nTotal: {}", valid, rejected, valid + rejected);
}

#[test]
fn test_replace_confusables() {
	let file = File::open("src/data/confusable_characters.txt");
	assert!(file.is_ok());

	let reader = BufReader::new(file.ok().unwrap());
	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();

		// The first character in `confusable_characters.txt` is the normalized character
		// that each subsequent character may be confused with
		let first_character = original_line.chars().next().unwrap();

		let normalized_line = replace_confusables(&original_line);
		for normalized_character in normalized_line.chars() {
			let normalized_character_codepoint =
				format!("\'\\u{{{:x}}}\'", normalized_character as u32);
			let first_character_codepoint = format!("\'\\u{{{:x}}}\'", first_character as u32);
			// println!("normalized_character_codepoint: {}  first_character_codepoint: {}", normalized_character_codepoint, first_character_codepoint);

			if consists_of_supported_unicode_character_sets(
				format!("{}{}", normalized_character, first_character).as_str(),
			) {
				assert_eq!(first_character_codepoint, normalized_character_codepoint);
			}
		}
	}
}

#[test]
fn test_strip_diacriticals() {
	let diacritical_string = "ÄÅÖäåöĂăĔĚĕĞğģĬĭŎŏŬǓŭàáâñ⁰⁴⁵₀₁₂ด้้้้้็็็็็้้้้้็็็็Z̮̞̠͙͔ͅḀ̗̞͈̻̗Ḷ͙͎̯̹̞͓G̻O̭̗̮𝕿";
	let stripped_string = strip_diacriticals(diacritical_string);
	assert_eq!(stripped_string, "AAOaaoAaEEeGggIiOoUUuaaan045012ดZALGOT");
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
		format_args!("{}hello{}world!{}", whitespace_string, whitespace_string, whitespace_string)
			.to_string();
	println!("String with whitespace: {}", string_with_whitespace);
	let whitespace_stripped_string = strip_unicode_whitespace(&string_with_whitespace);
	println!("Whitespace stripped string: {}", whitespace_stripped_string);
	assert_eq!(whitespace_stripped_string, "helloworld!");
}

#[test]
fn test_convert_to_canonical_combining_marks_reduce_to_the_same_canonical_form() {
	// Construct a handle "Álvaro" where the first character consists of
	// a base character and a combining mark for an accute accent
	let mut handle_with_combining_mark = String::new();
	handle_with_combining_mark.push('\u{0041}');
	handle_with_combining_mark.push('\u{0301}');
	handle_with_combining_mark.push_str("lvaro");

	// Construct the handle "Álvaro" where the first character consists
	// of the Latin-1 Supplement character 0x00C1, which contains both
	// the base character `A` and the accute diacritical in one character
	let mut handle_without_combining_mark = String::new();
	handle_without_combining_mark.push('\u{00C1}');
	handle_without_combining_mark.push_str("lvaro");

	let canonical_with_combining_mark = convert_to_canonical(&handle_with_combining_mark);
	let canonical_without_combining_mark = convert_to_canonical(&handle_without_combining_mark);

	let expected_canonical_form = String::from("a1var0");
	assert_eq!(canonical_with_combining_mark, canonical_without_combining_mark);
	assert_eq!(canonical_with_combining_mark, expected_canonical_form);
}

#[test]
fn test_split_display_name_success() {
	assert_eq!(split_display_name("hello.123"), Some((String::from("hello"), 123u16)));
	assert_eq!(split_display_name("hello.0"), Some((String::from("hello"), 0)));
	assert_eq!(split_display_name("español.123"), Some((String::from("español"), 123)));
	assert_eq!(split_display_name("日本語.123"), Some((String::from("日本語"), 123)));
}

#[test]
fn test_split_display_name_failure() {
	assert_eq!(split_display_name("hello123"), None);
	assert_eq!(split_display_name("hello.-123"), None);
	assert_eq!(split_display_name("hello.abc"), None);
	assert_eq!(split_display_name("hello.abc123"), None);
	assert_eq!(split_display_name("hello.12.3"), None);
	assert_eq!(split_display_name("hello."), None);
	assert_eq!(split_display_name("hello.0xffff"), None);
	// u16::MAX + 1
	assert_eq!(split_display_name("hello.65536"), None);
	assert_eq!(split_display_name("hello.999999999"), None);
}
