//! # Handle Validator
//!
//! `handle_validator` provides a `HandleValidator` struct to determine the validity of a given
//! user handle.

use core::ops::RangeInclusive;

/// Reserved words that cannot be used as the handle.
const RESERVED_WORDS: [&str; 8] =
	["admin", "everyone", "all", "administrator", "mod", "moderator", "here", "channel"];
/// Characters that cannot be used in the handle.
const BLOCKED_CHARACTERS: [char; 5] = ['@', '#', ':', '.', '`'];
/// Character that are allowed.
const ALLOWED_UNICODE_CHARACTER_RANGES: [RangeInclusive<u16>; 20] = [
	0x0020..=0x007F, // Basic Latin
	0x0080..=0x00FF, // Latin-1 Supplement
	0x0100..=0x017F, // Latin Extended-A
	0x1100..=0x11FF, // Hangul Jamo
	0xAC00..=0xD7AF, // Hangul Syllables
	0x30A0..=0x30FF, // Katakana
	0x3040..=0x309F, // Hiragana
	0x4E00..=0x9FFF, // CJK Unified Ideographs
	0x3400..=0x4DBF, // CJK Unified Ideographs Extension A
	0xF900..=0xFAFF, // CJK Compatibility Ideographs
	0x0980..=0x09FF, // Bengali
	0x0900..=0x097F, // Devanagari
	0x0400..=0x04FF, // Cyrillic
	0x0500..=0x052F, // Cyrillic Supplementary
	0x0370..=0x03FF, // Greek and Coptic
	0x1F00..=0x1FFF, // Greek Extended
	0x0E00..=0x0E7F, // Thai
	0x0600..=0x06FF, // Arabic
	0xFB50..=0xFDFF, // Arabic Presentation Forms-A
	0x0590..=0x05FF, // Hebrew
];

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
	input_str.chars().any(|c| BLOCKED_CHARACTERS.contains(&c))
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
