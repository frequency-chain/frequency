use std::{
	fs::File,
	io::{BufRead, BufReader, Write},
};

// Helper function to generate a BTreeMap which maps each confusable character to its
// canonical counterpart for each line in `confusable_characters.txt`.
//
// The first character of each line represent the canonical character (value) in the map that each
// subsequent character of the line (key) maps to.
//
// Usage:
//  confusables_generator
//
// Note that the executable must be run in the same folder as the data file, confusable_characters.txt.
// Running this tool generates a Rust source file called "confusables.rs"
fn convert_confuseables_to_unicode_escaped() {
	let input_file = File::open("confusable_characters.txt");
	assert!(input_file.is_ok());

	let mut output_file = std::fs::File::create("confusables.rs").expect("create failed");

	let reader = BufReader::new(input_file.ok().unwrap());

	_ = output_file.write_all(b"// ******************************************************\n");
	_ = output_file.write_all(b"// ***** THIS FILE IS AUTO-GENERATED.  DO NOT EDIT! *****\n");
	_ = output_file.write_all(b"// ******************************************************\n");
	_ = output_file.write_all(b"\n\n");
	_ = output_file.write_all(b"use phf::phf_map;\n");
	_ = output_file.write_all(b"pub static CONFUSABLES: phf::Map<char, char> = phf_map! {\n");

	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();
		let mut original_line_characters = original_line.chars();

		// The first character in the line will be the value of each key
		let normalized_character = original_line_characters.next().unwrap();

		while let Some(homoglyph) = original_line_characters.next() {
			let line = format!(
				"\'\\u{{{:x}}}\' => \'\\u{{{:x}}}\',\n",
				homoglyph as u32, normalized_character as u32
			);
			_ = output_file.write_all(line.as_bytes());
		}
	}
	_ = output_file.write_all(b"};\n\n");

	println!("data written to confusables.rs");
}

fn main() {
	convert_confuseables_to_unicode_escaped();
}
