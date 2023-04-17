//! # Suffix Generator
//!
//! `suffix_generator` provides a `SuffixGenerator` struct to generate unique suffix sequences for a given range
//! and seed, excluding already used suffixes.

use oorandom::Rand32;
use twox_hash::XxHash64;
extern crate alloc;
use alloc::vec::Vec;
use core::hash::Hasher;
/// Generate a unique, shuffled suffix iterator.
///
/// # Returns
///
/// An iterator over the unique, shuffled sequence of suffixes.
///
/// # Examples
///
/// ```
/// let min = 100;
/// let max = 150;
/// let canonical_base = "myhandle";
///
/// let lazy_sequence = handles_utils::suffix::generate_unique_suffixes(min, max, canonical_base);
/// let suffixes: Vec<u16> = lazy_sequence.collect();
/// ```
///
/// This will output a unique, shuffled sequence of suffixes.
/// Note: This is a lazy iterator, so it will not be evaluated until it is consumed.
pub fn generate_unique_suffixes(
	min: u16,
	max: u16,
	canonical_base: &str,
) -> impl Iterator<Item = u16> + '_ {
	let seed = generate_seed(canonical_base);
	let mut rng = Rand32::new(seed);

	let mut indices: Vec<u16> = (min..=max).collect();
	// let min = min as usize;
	// let max = max as usize;
	(min..=max).rev().map(move |i| {
		let j = rng.rand_range((min as u32)..(i + 1) as u32) as u16;
		indices.swap((i - min) as usize, (j - min) as usize);
		indices[(i - min) as usize]
	})
}

/// Generate a seed from a unique canonical base handle.
///
/// # Arguments
///
/// * `canonical_base` - The canonical base as a string slice.
///
/// # Returns
///
/// A 64-bit seed.
///
/// # Examples
/// ```
/// let canonical_base = "myuser";
///
/// let seed = handles_utils::suffix::generate_seed(canonical_base);
/// ```
pub fn generate_seed(canonical_base: &str) -> u64 {
	let mut hasher = XxHash64::with_seed(0);
	hasher.write(canonical_base.as_bytes());
	let value_bytes: [u8; 4] = [0; 4];
	hasher.write(&value_bytes);
	hasher.finish()
}
