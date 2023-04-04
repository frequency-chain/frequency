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
// `rustc utils.rs`
// `./utils`
//
// outputs to confusables.rs
fn convert_confuseables_to_unicode_escaped() {
	let input_file = File::open("confusable_characters.txt");
	assert!(input_file.is_ok());

	let mut output_file = std::fs::File::create("confusables.rs").expect("create failed");

	let reader = BufReader::new(input_file.ok().unwrap());

	output_file.write_all("// ******************************************************\n\n".as_bytes());
	output_file.write_all("// ***** THIS FILE IS AUTO-GENERATED.  DO NOT EDIT! *****\n\n".as_bytes());
	output_file.write_all("// ******************************************************\n\n".as_bytes());
	output_file.write_all("\n\n".as_bytes());
	output_file.write_all("use sp_std::collections::btree_map::BTreeMap;\n\n".as_bytes());
	output_file.write_all("pub fn build_confusables_map() -> BTreeMap<char, char> {\n".as_bytes());
	output_file.write_all("\tBTreeMap::from([\n".as_bytes());

	for line_result in reader.lines() {
		let original_line = line_result.ok().unwrap();
		let mut original_line_characters = original_line.chars();

		// The first character in the line will be the value of each key
		let normalized_character = original_line_characters.next().unwrap();

		while let Some(homoglyph) = original_line_characters.next() {
			let key = format!("\t\t(\'\\u{{{:x}}}\',", homoglyph as u32);
			output_file.write_all(key.as_bytes());

			let value = format!(" \'\\u{{{:x}}}\'),\n", normalized_character as u32);
			output_file.write_all(value.as_bytes());
		}
	}

	output_file.write_all("\t])\n".as_bytes());
	output_file.write_all("}\n".as_bytes());

	println!("data written to confusables.rs");
}

fn main() {
	convert_confuseables_to_unicode_escaped();
}
