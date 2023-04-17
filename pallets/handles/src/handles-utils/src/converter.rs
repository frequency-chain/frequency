//! # Handle Converter
//!
//! `handle_converter` provides functions to detect confusable Unicode characters in a
//! given input string and return its canonical form.
use crate::{confusables::CONFUSABLES, types::*};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
extern crate alloc;
use alloc::{
	string::{String, ToString},
	vec::Vec,
};

/// Delimiter for handles
pub const HANDLE_DELIMITER: char = '.';

/// Creates a new `HandleConverter` instance with a built confusables map.
/// Converts a given string to its canonical form by stripping Unicode whitespace,
/// replacing confusable characters, and stripping diacritical marks.
/// The resulting string is converted to lowercase ASCII characters.
///
/// # Arguments
///
/// * `input_str` - The input string to convert to canonical form.
///
/// # Returns
///
/// A new string in canonical form.
///
pub fn convert_to_canonical(input_str: &str) -> alloc::string::String {
	let white_space_stripped = strip_unicode_whitespace(input_str);
	let confusables_removed = replace_confusables(&white_space_stripped);
	let diacriticals_stripped = strip_diacriticals(&confusables_removed);
	diacriticals_stripped.to_ascii_lowercase()
}

/// Replaces any characters in the input string that are confusable with a different character.
///
/// # Arguments
///
/// * `input_str` - A reference to the input string to replace confusable characters.
///
/// # Returns
///
/// A new string where any confusable characters have been replaced with their corresponding non-confusable characters.
pub fn replace_confusables(input_str: &str) -> alloc::string::String {
	input_str
		.chars()
		.map(|character| CONFUSABLES.get(&character).map_or(character, |&value| value))
		.collect::<alloc::string::String>()
}

/// This function removes diacritical marks from the input string and returns a new `String` without them.
///
/// # Arguments
/// * `input_str` - A string slice that contains the input string from which the diacritical marks need to be removed.
///
/// # Notes
/// This function uses the NFKD normalization form and filtering of combining marks to strip the diacritical marks from the input string.
/// Combining marks are Unicode characters that are intended to modify the appearance of another character.
///
pub fn strip_diacriticals(input_str: &str) -> alloc::string::String {
	input_str
		.nfkd()
		.filter(|character| !is_combining_mark(*character))
		.collect::<alloc::string::String>()
}

/// Splits the given display name into its base handle and handle suffix.
///
/// # Arguments
///
/// * `display_name_str` - The display name to split.
///
/// # Returns
///
/// A tuple containing the base handle string and the handle suffix as a `HandleSuffix` enum.
///
pub fn split_display_name(display_name_str: &str) -> Option<(String, HandleSuffix)> {
	let parts: Vec<&str> = display_name_str.split(HANDLE_DELIMITER).collect();
	let base_handle_str = parts[0].to_string();
	if parts.len() != 2 {
		return None
	}

	let suffix = parts[1];
	let suffix_num = suffix.parse::<u16>().ok()?;

	Some((base_handle_str, suffix_num))
}

/// Strips any Unicode whitespace characters from the provided string and returns the resulting string.
///
/// # Arguments
///
/// * `input_str` - A string slice that holds the input string from which the Unicode whitespace characters need to be stripped.
///
/// # Returns
///
/// A new string without any Unicode whitespace characters.
pub fn strip_unicode_whitespace(input_str: &str) -> String {
	input_str
		.chars()
		.filter(|character| !character.is_whitespace())
		.collect::<alloc::string::String>()
}
