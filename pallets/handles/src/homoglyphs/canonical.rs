//! # Handle Converter
//!
//! `handle_converter` provides a `HandleConverter` struct to detect confusable Unicode characters in a
//! given input string and return its canonical form.
use crate::homoglyphs::confusables::build_confusables_map;
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
/// use pallet_handles::homoglyphs::canonical::HandleConverter;
///
/// let word = "ℂн𝔸RℒℰᏕ";
///
/// let handle_converter = HandleConverter::new();
/// let canonical_word = handle_converter.replace_confusables(word);
/// println!("{}", canonical_word);
///
/// ```
impl HandleConverter {
	/// Creates a new `HandleConverter` instance with the specified input string.
	pub fn new() -> Self {
		let confusables_map = build_confusables_map();
		Self { confusables_map }
	}
	/// Convert `string` to canonical form
	pub fn convert_to_canonical(&self, string: &str) -> codec::alloc::string::String {
		let confusables_removed = self.replace_confusables(string);
		let diacriticals_stripped = self.strip_diacriticals(&confusables_removed);
		diacriticals_stripped.to_ascii_lowercase()
		// 	normalized.make_ascii_lowercase();
	}

	/// Replace confusable Unicode characters from string
	pub fn replace_confusables(&self, string: &str) -> codec::alloc::string::String {
		string
			.chars()
			.map(|character| self.confusables_map.get(&character).map_or(character, |&value| value))
			.collect::<codec::alloc::string::String>()
	}

	/// Strip diacriticals (accent marks) from string
	pub fn strip_diacriticals(&self, string: &str) -> codec::alloc::string::String {
		string
			.nfkd()
			.filter(|character| !is_combining_mark(*character))
			.collect::<codec::alloc::string::String>()
	}

	/// Split display name into name and suffix
	pub fn split_display_name(&self, display_name_str: &str) -> (String, HandleSuffix) {
		let parts: Vec<&str> = display_name_str.split(".").collect();
		let base_handle_str = parts[0].to_string();
		let suffix = parts[1];
		let suffix_num = suffix.parse::<u16>().unwrap();
		(base_handle_str, suffix_num)
	}

	// /// Combine name, delimiter and suffix into display name
	// pub fn combine_display_name(&self, base_handle:Handle, delimeter:&str, suffix:u16) -> String {
	// }

	// /// Convert Handle (BoundedVec) to UTF-8 &str
	// pub fn convert_handle_to_string() -> String {
	// }

	/// Convert string to Handle
	pub fn convert_str_to_handle(input_str: &str) -> Handle {
		let input_vec = input_str.as_bytes().to_vec();
		let handle: Handle = input_vec.try_into().unwrap();
		handle
	}
	// /// Convert Vec<u8> into Handle (BoundedVec)
	// pub fn convert_vec_to_handle() -> Handle {
	// }
}
