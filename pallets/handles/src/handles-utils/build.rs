use std::{
	env,
	fs::File,
	io::{BufRead, BufReader, BufWriter, Write},
	path::Path,
};

mod constants;
use constants::ALLOWED_UNICODE_CHARACTER_RANGES;

fn allowed_char(c: char) -> bool {
	ALLOWED_UNICODE_CHARACTER_RANGES.iter().any(|range| range.contains(&(c as u16)))
}

fn main() {
	let input_file = File::open("src/data/confusable_characters.txt");
	assert!(input_file.is_ok());
	let path = Path::new(&env::var("OUT_DIR").unwrap()).join("confusables.rs");
	let mut output_file = BufWriter::new(File::create(&path).unwrap());

	let reader = BufReader::new(input_file.ok().unwrap());

	_ = output_file
		.write_all(b"// This module provides utilities for handling confusable characters.\n\n");
	_ = output_file.write_all(b"// ******************************************************\n");
	_ = output_file.write_all(b"// ***** THIS FILE IS AUTO-GENERATED.  DO NOT EDIT! *****\n");
	_ = output_file.write_all(b"// ******************************************************\n");
	_ = output_file.write_all(b"\n\n");
	_ = output_file.write_all(b"/// Map it\n");
	_ = output_file.write_all(b"use phf::phf_map;\n");
	_ = output_file
		.write_all(b"/// The mapping from homoglyph character to canonical Unicode character\n");
	_ = output_file.write_all(b"pub static CONFUSABLES: phf::Map<char, char> = phf_map! {\n");

	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();
		let mut original_line_characters = original_line.chars();

		// The first character in the line will be the value of each key
		let normalized_character = original_line_characters.next().unwrap();

		// Skip if the normalized is not in the allowed range
		if !allowed_char(normalized_character) {
			continue
		}

		while let Some(homoglyph) = original_line_characters.next() {
			// Skip if the homoglyph is not in the allowed range
			if allowed_char(homoglyph) {
				let line = format!(
					"\'\\u{{{:x}}}\' => \'\\u{{{:x}}}\',\n",
					homoglyph as u32, normalized_character as u32
				);
				_ = output_file.write_all(line.as_bytes());
			}
		}
	}
	_ = output_file.write_all(b"};\n\n");

	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=data/confusable_characters.txt");
	println!("cargo:rerun-if-changed=constants.rs");
}
