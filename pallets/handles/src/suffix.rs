//! # Suffix Generator
//!
//! `suffix_generator` provides a `SuffixGenerator` struct to generate unique suffix sequences for a given range
//! and seed, excluding already used suffixes.

use core::hash::Hasher;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sp_std::vec::Vec;
use twox_hash::XxHash64;

/// A generator for unique suffix sequences.
///
/// Given a min, max range, and a seed, generates unique suffix sequences by excluding
/// already used suffixes.
pub struct SuffixGenerator {
	min: usize,
	max: usize,
	rng: SmallRng,
}

impl SuffixGenerator {
	/// Creates a new `SuffixGenerator` instance with the specified min, max range and seed.
	///  
	/// # Arguments
	///
	/// * `min` - The minimum value of the range.
	/// * `max` - The maximum value of the range.
	/// * `canonical_handle` - The canonical handle as a string slice.
	///
	/// # Returns
	///
	/// A new `SuffixGenerator` instance.
	///
	/// # Example
	///
	/// ```
	/// use pallet_handles::suffix::SuffixGenerator;
	///
	/// let min = 100;
	/// let max = 150;
	/// let canonical_handle = "myhandle";
	///
	/// let suffix_generator = SuffixGenerator::new(min, max, "myhandle");
	/// ```
	pub fn new(min: usize, max: usize, canonical_handle: &str) -> Self {
		let seed = Self::generate_seed(canonical_handle);
		let rng = SmallRng::seed_from_u64(seed);
		Self { min, max, rng }
	}

	/// Generate a unique, shuffled suffix iterator.
	///
	/// # Returns
	///
	/// An iterator over the unique, shuffled sequence of suffixes.
	///
	/// # Examples
	///
	/// ```
	/// use pallet_handles::suffix::SuffixGenerator;
	///
	/// let min = 100;
	/// let max = 150;
	/// let canonical_handle = "myhandle";
	///
	/// let mut suffix_generator = SuffixGenerator::new(min, max, canonical_handle);
	///
	/// let sequence: Vec<usize> = suffix_generator.suffix_iter().take(10).collect();
	/// println!("{:?}", sequence);
	/// ```
	///
	/// This will output a unique, shuffled sequence of suffixes.
	pub fn suffix_iter(&mut self) -> impl Iterator<Item = usize> + '_ {
		let mut indices: Vec<usize> = (self.min..=self.max).collect();
		(self.min..=self.max).rev().map(move |i| {
			let j = self.rng.gen_range(self.min..=i);
			indices.swap(i - self.min, j - self.min);
			indices[i - self.min]
		})
	}

	/// Generate a seed from a unique canonical base handle.
	///
	/// # Arguments
	///
	/// * `canonical_handle` - The canonical handle as a string slice.
	///
	/// # Returns
	///
	/// A 64-bit seed.
	///
	/// # Examples
	/// ```
	/// use pallet_handles::suffix::SuffixGenerator;
	///
	/// let canonical_handle = "myuser";
	///
	/// let seed = SuffixGenerator::generate_seed(canonical_handle);
	/// ```
	pub fn generate_seed(canonical_handle: &str) -> u64 {
		let mut hasher = XxHash64::with_seed(0);
		sp_std::hash::Hash::hash(&canonical_handle, &mut hasher);
		let value_bytes: [u8; 4] = [0; 4];
		hasher.write(&value_bytes);
		hasher.finish()
	}
}
