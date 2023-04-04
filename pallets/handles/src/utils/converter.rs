//! # Handle Converter
//!
//! `handle_converter` provides a `HandleConverter` struct to detect confusable Unicode characters in a
//! given input string and return its canonical form.
use crate::utils::confusables::build_confusables_map;
use common_primitives::handles::*;
use sp_std::collections::btree_map::BTreeMap;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
extern crate alloc;
use alloc::string::{String, ToString};
use sp_std::vec::Vec;
/// A converter for confusable characters.
///
/// Given a string, detects easily confusable characters and returns the string in canonical form.
pub struct HandleConverter {
	confusables_map: BTreeMap<char, char>,
}
/// Creates a new `HandleConverter` instance with the specified input string.
///
/// # Example
///
/// ```
/// use pallet_handles::utils::converter::HandleConverter;
///
/// let word = "ℂн𝔸RℒℰᏕ";
///
/// let handle_converter = HandleConverter::new();
/// let canonical_word = handle_converter.replace_confusables(word);
/// println!("{}", canonical_word);
///
/// ```
impl HandleConverter {
	/// Creates a new `HandleConverter` instance with a built confusables map.
	pub fn new() -> Self {
		let confusables_map = build_confusables_map();
		Self { confusables_map }
	}
	/// Converts a given string to its canonical form by stripping Unicode whitespace,
	/// replacing confusable characters, and stripping diacritical marks.
	/// The resulting string is converted to lowercase ASCII characters.
	///
	/// # Arguments
	///
	/// * `self` - The `Canonicalizer` struct instance.
	/// * `string` - The input string to be converted.
	///
	/// # Returns
	///
	/// The canonicalized string.
	///
	/// # Examples
	///
	/// ```
	/// use canonicalizer::Canonicalizer;
	///
	/// let c = Canonicalizer::new();
	/// let input = "Héllo, Wörld!";
	/// let canonicalized = c.convert_to_canonical(input);
	/// assert_eq!(canonicalized, "hello, world!");
	/// ```
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
	/// * `string` - A reference to the input string to replace confusable characters.
	///
	/// # Returns
	///
	/// A new string where any confusable characters have been replaced with their corresponding non-confusable characters.
	///
	/// # Example
	///
	/// ```
	/// use confusable_detector::ConfusableDetector;
	///
	/// let detector = ConfusableDetector::new();
	///
	/// let input = "𝕙𝕖𝕝𝕝𝕠 𝕨𝕠𝕣𝕝𝕕"; // Contains homoglyph characters
	///
	/// let output = detector.replace_confusables(&input);
	///
	/// assert_eq!(output, "hello world");
	/// ```
	pub fn replace_confusables(&self, string: &str) -> codec::alloc::string::String {
		string
			.chars()
			.map(|character| self.confusables_map.get(&character).map_or(character, |&value| value))
			.collect::<codec::alloc::string::String>()
	}

	/// This function removes diacritical marks from the input string and returns a new `String` without them.
	///
	/// # Arguments
	/// * `string` - A string slice that contains the input string from which the diacritical marks need to be removed.
	///
	/// # Example
	/// ```
	/// use my_crate::strip_diacriticals;
	///
	/// let input_string = "pâté";
	/// let expected_output = "pate";
	/// let output_string = strip_diacriticals(input_string);
	/// assert_eq!(expected_output, output_string);
	/// ```
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
	/// # Examples
	///
	/// ```
	/// use my_crate::HandleSuffix;
	/// let (base_handle_str, suffix) = split_display_name("my_handle.123");
	/// assert_eq!(base_handle_str, "my_handle");
	/// assert_eq!(suffix, HandleSuffix::Number(123));
	/// ```
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
	/// # Example
	///
	/// ```
	/// use my_crate::convert_str_to_handle;
	///
	/// let input_str = "my_string";
	/// let handle = convert_str_to_handle(input_str);
	/// ```
	///
	/// # Returns
	///
	/// A handle that can be used to interact with other parts of the codebase that
	/// require a handle.
	///
	pub fn convert_str_to_handle(input_str: &str) -> Handle {
		let input_vec = input_str.as_bytes().to_vec();
		let handle: Handle = input_vec.try_into().unwrap();
		handle
	}

	/// Strips any Unicode whitespace characters from the provided string and returns the resulting string.
	///
	/// # Arguments
	///
	/// * `string` - A reference to a string slice that contains the input string to be stripped of Unicode whitespace.
	///
	/// # Examples
	///
	/// ```
	/// use my_crate::string_utils;
	///
	/// let input = "Hello,  \t world!\n";
	/// let expected_output = "Hello,world!";
	///
	/// assert_eq!(string_utils::strip_unicode_whitespace(input), expected_output);
	/// ```

	pub fn strip_unicode_whitespace(&self, string: &str) -> String {
		string
			.chars()
			.filter(|character| !character.is_whitespace())
			.collect::<codec::alloc::string::String>()
	}
}
