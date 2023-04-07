//! # Handle Converter
//!
//! `handle_converter` provides a `HandleConverter` struct to detect confusable Unicode characters in a
//! given input string and return its canonical form.
use crate::utils::confusables::CONFUSABLES;
use common_primitives::handles::*;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
extern crate alloc;
use alloc::string::{String, ToString};
use sp_std::vec::Vec;
/// A converter for confusable characters.
///
/// Given a string, detects easily confusable characters and returns the string in canonical form.
pub struct HandleConverter {}
/// Creates a new `HandleConverter` instance with the specified input string.
impl HandleConverter {
	/// Creates a new `HandleConverter` instance with a built confusables map.
	pub fn new() -> Self {
		Self {}
	}
	/// Converts a given string to its canonical form by stripping Unicode whitespace,
	/// replacing confusable characters, and stripping diacritical marks.
	/// The resulting string is converted to lowercase ASCII characters.
	///
	/// # Arguments
	///
	/// * `self` - A reference to a `ConfusableDetector` instance.
	/// * `String` - A reference to the input string to convert to canonical form.
	///
	/// # Returns
	///
	/// A new string in canonical form.
	///
	pub fn convert_to_canonical(&self, string: &str) -> codec::alloc::string::String {
		let white_space_stripped = self.strip_unicode_whitespace(string);
		let confusables_removed = self.replace_confusables(&white_space_stripped);
		let diacriticals_stripped = self.strip_diacriticals(&confusables_removed);
		diacriticals_stripped.to_ascii_lowercase()
	}

	/// Replaces any characters in the input string that are confusable with a different character.
	///
	/// # Arguments
	///
	/// * `self` - A reference to a `ConfusableDetector` instance.
	/// * `String` - A reference to the input string to replace confusable characters.
	///
	/// # Returns
	///
	/// A new string where any confusable characters have been replaced with their corresponding non-confusable characters.
	pub fn replace_confusables(&self, string: &str) -> codec::alloc::string::String {
		string
			.chars()
			.map(|character| CONFUSABLES.get(&character).map_or(character, |&value| value))
			.collect::<codec::alloc::string::String>()
	}

	/// This function removes diacritical marks from the input string and returns a new `String` without them.
	///
	/// # Arguments
	/// * `String` - A string slice that contains the input string from which the diacritical marks need to be removed.
	///
	/// # Notes
	/// This function uses the NFKD normalization form and filtering of combining marks to strip the diacritical marks from the input string.
	/// Combining marks are Unicode characters that are intended to modify the appearance of another character.
	///
	pub fn strip_diacriticals(&self, string: &str) -> codec::alloc::string::String {
		string
			.nfkd()
			.filter(|character| !is_combining_mark(*character))
			.collect::<codec::alloc::string::String>()
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
	pub fn split_display_name(&self, display_name_str: &str) -> (String, HandleSuffix) {
		let parts: Vec<&str> = display_name_str.split(".").collect();
		let base_handle_str = parts[0].to_string();
		let suffix = parts[1];
		let suffix_num = suffix.parse::<u16>().unwrap();
		(base_handle_str, suffix_num)
	}

	/// Converts a string into a handle.
	///
	/// # Arguments
	///
	/// * `input_str` - A string slice that holds the input to be converted.
	///
	/// # Returns
	///
	/// A `Handle` instance.
	pub fn convert_str_to_handle(input_str: &str) -> Handle {
		let input_vec = input_str.as_bytes().to_vec();
		let handle: Handle = input_vec.try_into().unwrap();
		handle
	}

	/// Strips any Unicode whitespace characters from the provided string and returns the resulting string.
	///
	/// # Arguments
	///
	/// * `String` - A reference to a string slice that contains the input string to be stripped of Unicode whitespace.
	pub fn strip_unicode_whitespace(&self, string: &str) -> String {
		string
			.chars()
			.filter(|character| !character.is_whitespace())
			.collect::<codec::alloc::string::String>()
	}
}
