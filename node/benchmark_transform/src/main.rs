/// This tool reads the generated weight.rs file that includes child trie access and replaces
/// these DB Read and Writes with child trie specific DB weights located at rocksdb_child_trie_weights.rs
use regex::Regex;
use std::{
	env, fs,
	io::{self},
	process,
};

fn main() -> io::Result<()> {
	// Get command line arguments
	let args: Vec<String> = env::args().collect();

	if args.len() != 3 {
		eprintln!("Usage: {} <input_weights_file> <output_weights_file>", args[0]);
		eprintln!("Example: {} weights.rs transformed_weights.rs", args[0]);
		process::exit(1);
	}

	let input_file = &args[1];
	let output_file = &args[2];

	// Read the input file
	let input = fs::read_to_string(input_file).map_err(|e| {
		eprintln!("Error reading file '{input_file}': {e}");
		e
	})?;

	// Check if already modified
	if is_already_modified(&input) {
		println!("✓ It was already modified and no need for transformation!");
		return Ok(())
	}

	// Transform the content
	let output = transform_weights(&input);

	// Write to output file
	fs::write(output_file, &output).map_err(|e| {
		eprintln!("Error writing to file '{output_file}': {e}");
		e
	})?;

	println!("✓ Transformation complete!");
	println!("  Input:  {input_file}");
	println!("  Output: {output_file}");
	Ok(())
}

fn is_already_modified(content: &str) -> bool {
	content.contains("'benchmark_transform'")
}

fn transform_weights(content: &str) -> String {
	let mut result = content.to_string();

	// Step 1: Adds the modification disclaimer
	result = ["\n//! MODIFIED by 'benchmark_transform' tool to replace child trie storage access with their specific DB weights", &result].join("\n//!");

	// Step 2: Add RocksDbWeightChild import after the PhantomData import
	let import_pattern = "use core::marker::PhantomData;";
	let new_import = "use core::marker::PhantomData;\nuse common_runtime::weights::rocksdb_child_trie_weights::constants::RocksDbWeightChild;";
	result = result.replace(import_pattern, new_import);

	// Step 3: Process each function in SubstrateWeight<T> and the () implementation
	result = process_weight_implementations(&result);

	result
}

fn process_weight_implementations(content: &str) -> String {
	let mut result = String::new();
	let lines: Vec<&str> = content.lines().collect();
	let mut i = 0;

	while i < lines.len() {
		let line = lines[i];
		result.push_str(line);
		result.push('\n');

		// Check if we're at a function definition
		if line.trim().starts_with("fn ") && line.contains("-> Weight {") {
			// Collect function body lines and storage comments
			let mut func_lines = vec![];
			let mut storage_lines = Vec::new();

			// Look backwards for Storage comments
			let mut j = i;
			while j > 0 {
				j -= 1;
				let prev_line = lines[j];
				if prev_line.trim().starts_with("/// Storage:") {
					storage_lines.insert(0, prev_line);
				} else if !prev_line.trim().starts_with("///") {
					break;
				}
			}

			// Collect function body
			let mut brace_count = 0;
			let mut in_function = false;

			while i < lines.len() {
				let current = lines[i];
				func_lines.push(current);

				if current.contains('{') {
					in_function = true;
					brace_count += current.matches('{').count();
				}
				if current.contains('}') {
					brace_count -= current.matches('}').count();
				}

				if in_function && brace_count == 0 {
					break;
				}
				i += 1;
			}

			// Count UNKNOWN keys
			let unknown_reads = count_unknown_keys(&storage_lines, "r:");
			let unknown_writes = count_unknown_keys(&storage_lines, "w:");

			if unknown_reads > 0 || unknown_writes > 0 {
				// Process the function body
				let processed =
					process_function_body(&func_lines.join("\n"), unknown_reads, unknown_writes);
				// Remove the original function line since we already added it
				let processed_lines: Vec<&str> = processed.lines().collect();
				// Skip first line as it's already added
				for pline in &processed_lines[1..] {
					result.push_str(pline);
					result.push('\n');
				}
				i += 1;
				continue;
			}
		}

		i += 1;
	}

	result
}

fn count_unknown_keys(storage_lines: &[&str], operation: &str) -> u64 {
	let mut count = 0;
	for line in storage_lines {
		if line.contains("UNKNOWN KEY") && line.contains(operation) {
			// Extract the operation count (e.g., "r:1" or "w:1")
			if let Some(idx) = line.find(operation) {
				let after = &line[idx + operation.len()..];
				if let Some(space_idx) = after.find(|c: char| c.is_whitespace() || c == ')') {
					if let Ok(num) = after[..space_idx].parse::<u64>() {
						count += num;
					}
				}
			}
		}
	}
	count
}

fn process_function_body(func_body: &str, unknown_reads: u64, unknown_writes: u64) -> String {
	let lines: Vec<&str> = func_body.lines().collect();
	let mut result = Vec::new();
	let mut i = 0;
	let read_regex = Regex::new(r"reads\((\d+)_u64\)").expect("Should create regex.");
	let write_regex = Regex::new(r"writes\((\d+)_u64\)").expect("Should create regex.");

	while i < lines.len() {
		let line = lines[i];

		// Check for T::DbWeight::get().reads() or RocksDbWeight::get().reads()
		if (line.contains("T::DbWeight::get().reads(") ||
			line.contains("RocksDbWeight::get().reads(")) &&
			unknown_reads > 0
		{
			// Extract the current read count
			if let Some(caps) = read_regex.captures(line) {
				if let Ok(current_reads) = caps[1].parse::<u64>() {
					// Only subtract if current_reads is greater than unknown_reads
					if current_reads >= unknown_reads {
						let new_reads = current_reads - unknown_reads;
						let indent = get_indent(line);

						// Determine which DbWeight to use
						let db_weight = if line.contains("T::DbWeight") {
							"T::DbWeight"
						} else {
							"RocksDbWeight"
						};

						// Add the modified line
						if new_reads > 0 {
							result.push(format!(
								"{indent}.saturating_add({db_weight}::get().reads({new_reads}_u64))"
							));
						}
						// Add the child trie reads line
						result.push(format!(
							"{indent}.saturating_add(RocksDbWeightChild::get().reads({unknown_reads}_u64))"
						));
						i += 1;
						continue;
					}
				}
			}
		}

		// Check for T::DbWeight::get().writes() or RocksDbWeight::get().writes()
		if (line.contains("T::DbWeight::get().writes(") ||
			line.contains("RocksDbWeight::get().writes(")) &&
			unknown_writes > 0
		{
			if let Some(caps) = write_regex.captures(line) {
				if let Ok(current_writes) = caps[1].parse::<u64>() {
					// Only subtract if current_writes is greater than unknown_writes
					if current_writes >= unknown_writes {
						let new_writes = current_writes - unknown_writes;
						let indent = get_indent(line);

						let db_weight = if line.contains("T::DbWeight") {
							"T::DbWeight"
						} else {
							"RocksDbWeight"
						};

						if new_writes > 0 {
							result.push(format!(
								"{indent}.saturating_add({db_weight}::get().writes({new_writes}_u64))"
							));
						}
						result.push(format!(
							"{indent}.saturating_add(RocksDbWeightChild::get().writes({unknown_writes}_u64))"
						));
						i += 1;
						continue;
					}
				}
			}
		}

		result.push(line.to_string());
		i += 1;
	}

	result.join("\n")
}

fn get_indent(line: &str) -> String {
	let trimmed = line.trim_start();
	line[..line.len() - trimmed.len()].to_string()
}
