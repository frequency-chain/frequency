//! # Handle Validator
//!
//! `handle_validator` provides a `HandleValidator` struct to determine the validity of a given
//! user handle.
use sp_std::prelude::Vec;

/// A validator for user handles.
///
/// Given a String user handle as input, determines if the handle is valid.
pub struct HandleValidator<'string_lifetime> {
	reserved_handles: Vec<&'string_lifetime str>,
	blocked_characters: Vec<char>,
}

impl<'string_lifetime> HandleValidator<'string_lifetime> {
	/// Create a new, intialized `HandleValidator` instance.
	pub fn new() -> Self {
		// This intialization data *could* be passed into the constructor and set as a pallet constant
		let reserved_handles: Vec<&str> = Vec::from(["admin", "everyone", "all"]);
		let blocked_characters: Vec<char> = Vec::from(['@', '#', ':', '.', '`']);

		Self { reserved_handles, blocked_characters }
	}

	/// Determines whether a given string is a reserved handle in the current context.
	///
	/// # Arguments
	///
	/// * `string` - A string slice representing the handle to check.
	///
	/// # Returns
	///
	/// A boolean value indicating whether the string is a reserved handle (`true`) or not (`false`).
	pub fn is_reserved_handle(&self, string: &str) -> bool {
		for handle in &self.reserved_handles {
			if &string == handle {
				return true
			}
		}
		false
	}

	/// Checks if the given string contains any blocked characters.
	///
	/// # Arguments
	///
	/// * `string` - A string slice to check for blocked characters.
	pub fn contains_blocked_characters(&self, string: &str) -> bool {
		for character in &self.blocked_characters {
			if string.contains(|c| c == *character) {
				return true
			}
		}
		false
	}
}
