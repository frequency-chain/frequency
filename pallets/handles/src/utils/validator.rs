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

	/// Determines if the given string exists in reserved_handles.
	pub fn is_reserved_handle(&self, string: &str) -> bool {
		for handle in &self.reserved_handles {
			if &string == handle {
				return true
			}
		}
		false
	}

	/// Determines if the given string contains a character in blocked_characters.
	pub fn contains_blocked_characters(&self, string: &str) -> bool {
		for character in &self.blocked_characters {
			if string.contains(|c| c == *character) {
				return true
			}
		}
		false
	}
}
