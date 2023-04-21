use crate::validator::{
	consists_of_supported_unicode_character_sets, contains_blocked_characters, is_reserved_handle,
	is_valid,
};

use crate::convert_to_canonical;

#[test]
fn test_is_reserved_handle_happy_path() {
	let reserved_handles: Vec<&str> =
		vec!["admin", "everyone", "all", "mod", "moderator", "administrator", "here", "channel"];

	for handle in reserved_handles {
		assert!(is_reserved_handle(handle));
	}
}

#[test]
fn test_is_reserved_handle_negative() {
	let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont"];
	for handle in handles {
		assert!(!is_reserved_handle(handle));
	}
}

#[test]
fn test_contains_blocked_characters_happy_path() {
	let handles: Vec<&str> =
		vec!["@lbert", "coca:cola", "#freemont", "charles.darwin", "`String`", ":(){ :|:& };:"];
	for handle in handles {
		assert!(contains_blocked_characters(handle));
	}
}

#[test]
fn test_contains_blocked_characters_negative() {
	let handles: Vec<&str> =
		vec!["albert", "coca_cola", "freemont", "charles-darwin", "Single Quote'd"];
	for handle in handles {
		assert!(!contains_blocked_characters(handle));
	}
}

#[test]
fn test_consists_of_supported_unicode_character_sets_happy_path() {
	let strings_containing_characters_in_supported_unicode_character_sets = Vec::from([
		"John",                  // Basic Latin
		"Álvaro",               // Latin-1 Supplement
		"가영",                // Hangul Syllables
		"가나다",             // Hangul Syllables
		"アキラ",             // Katakana
		"あいこ",             // Hiragana
		"李明",                // CJK Unified Ideographs
		"严勇",                // CJK Unified Ideographs
		"龍",                   // CJK Unified Ideographs
		"অমিত",          // Bengali
		"आरव",             // Devanagari
		"Александр",    // Cyrillic
		"Αλέξανδρος",  // Greek and Coptic
		"Ἀναξαγόρας", // Greek Extended
		"กัญญา",       // Thai
		"عمر",                // Arabic
		"דָּנִיֵּאל",  // Hewbrew
	]);

	for string in strings_containing_characters_in_supported_unicode_character_sets {
		assert!(consists_of_supported_unicode_character_sets(string));
	}
}

#[test]
fn test_consists_of_supported_unicode_character_sets_rejects_emojis() {
	// Constructing a string that with the smiling face emoji
	let string_containing_emojis = format_args!("John{}", '\u{1F600}').to_string();

	assert!(!consists_of_supported_unicode_character_sets(&string_containing_emojis));
}

// Will load `CONFUSABLES` with all the confusables at build time.
// See build.rs
include!(concat!(env!("OUT_DIR"), "/confusables.rs"));

#[test]
fn test_confusables_map_does_not_contain_keys_in_unsupported_character_sets() {
	for key in CONFUSABLES.keys() {
		assert!(consists_of_supported_unicode_character_sets(&key.to_string()));
	}
}

#[test]
fn wil_should_reject_symbols() {
	let should_reject = vec!["⏾", "☾", "🌘"];

	for s in should_reject {
		assert!(!consists_of_supported_unicode_character_sets(s));
	}
}

#[test]
fn wil_test_specific_one() {
	let canon = convert_to_canonical("13rukato24");
	for c in canon.chars() {
		if !is_valid(canon.as_str()) {
			println!("Unable to validate char: \"{}\"", c);
			println!("Blocked Char?: \"{}\"", contains_blocked_characters(c.to_string().as_str()));
		}
	}
}
