use crate::validator::HandleValidator;
extern crate alloc;
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

#[test]
fn test_is_reserved_handle_happy_path() {
	let reserved_handles: Vec<&str> = vec!["admin", "everyone", "all"];
	let handle_validator = HandleValidator::new();

	for handle in reserved_handles {
		assert!(handle_validator.is_reserved_handle(handle));
	}
}

#[test]
fn test_is_reserved_handle_negative() {
	let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont"];
	let handle_validator = HandleValidator::new();
	for handle in handles {
		assert!(!handle_validator.is_reserved_handle(handle));
	}
}

#[test]
fn test_contains_blocked_characters_happy_path() {
	let handles: Vec<&str> = vec!["@lbert", "coca:cola", "#freemont", "charles.darwin", "`String`"];
	let handle_validator = HandleValidator::new();
	for handle in handles {
		assert!(handle_validator.contains_blocked_characters(handle));
	}
}

#[test]
fn test_contains_blocked_characters_negative() {
	let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont", "charles-darwin", "'String'"];
	let handle_validator = HandleValidator::new();
	for handle in handles {
		assert!(!handle_validator.contains_blocked_characters(handle));
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

	let handle_validator = HandleValidator::new();
	for string in strings_containing_characters_in_supported_unicode_character_sets {
		assert!(handle_validator.consists_of_supported_unicode_character_sets(string));
	}
}

#[test]
fn test_consists_of_supported_unicode_character_sets_rejects_emojis() {
	let handle_validator = HandleValidator::new();

	// Constructing a string that with the smiling face emoji
	let string_containing_emojis = format_args!("John{}", '\u{1F600}').to_string();

	assert!(
		!handle_validator.consists_of_supported_unicode_character_sets(&string_containing_emojis)
	);
}
