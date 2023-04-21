//! # Handle Validator
//!
//! `handle_validator` provides a `HandleValidator` struct to determine the validity of a given
//! user handle.

// We need the ALLOWED_UNICODE_CHARACTER_RANGES for the build
#[path = "../constants.rs"]
mod constants;
use constants::ALLOWED_UNICODE_CHARACTER_RANGES;

/// Reserved words that cannot be used as the handle.
const RESERVED_WORDS: [&str; 8] =
	["admin", "everyone", "all", "administrator", "mod", "moderator", "here", "channel"];

/// Characters that cannot be used in the handle.
const BLOCKED_CHARACTERS: [char; 16] =
	['"', '#', '%', '(', ')', ',', '.', ':', ';', '<', '>', '@', '\\', '`', '{', '}'];

// We MUST have the BLOCKED_CHARACTERS constant sorted or we cannot use the faster `binary_search` function.
// Cannot easily be sorted at compile time currently
#[test]
fn ensure_sorted_blocked_characters() {
	let mut sorted = BLOCKED_CHARACTERS;
	sorted.sort();
	assert_eq!(BLOCKED_CHARACTERS, sorted);
}


/// Determines whether a given string is a reserved handle in the current context.
///
/// # Arguments
///
/// * `input_str` - A string slice representing the handle to check.
///
/// # Returns
///
/// A boolean value indicating whether the string is a reserved handle (`true`) or not (`false`).
pub fn is_reserved_handle(input_str: &str) -> bool {
	RESERVED_WORDS.contains(&input_str)
}

/// Checks if the given string contains any blocked characters.
///
/// # Arguments
///
/// * `input_str` - A string slice to check for blocked characters.\
///
/// # Returns
///
/// A boolean value indicating whether the string contains any blocked characters (`true`) or not (`false`).
pub fn contains_blocked_characters(input_str: &str) -> bool {
	input_str.chars().any(|c| BLOCKED_CHARACTERS.binary_search(&c).is_ok())
}

/// Checks that the given string contains characters within the ranges of supported
/// unicode character sets.
///
/// # Arguments
///
/// * `input_str` - A string slice to check for characters within the allowed unicode character sets..
///
/// # Returns
///
/// A boolean value indicating whether the string contains characters within the allowed unicode character sets (`true`) or not (`false`).
pub fn consists_of_supported_unicode_character_sets(input_str: &str) -> bool {
	input_str
		.chars()
		.all(|c| ALLOWED_UNICODE_CHARACTER_RANGES.iter().any(|range| range.contains(&(c as u16))))
}
