// We need the ALLOWED_UNICODE_CHARACTER_RANGES for the build as well as in the source, so here it is.

use core::ops::RangeInclusive;

/// Character that are allowed.
pub const ALLOWED_UNICODE_CHARACTER_RANGES: [RangeInclusive<u16>; 20] = [
	0x0020..=0x007F, // Basic Latin
	0x0080..=0x00FF, // Latin-1 Supplement
	0x0100..=0x017F, // Latin Extended-A
	0x1100..=0x11FF, // Hangul Jamo
	0xAC00..=0xD7AF, // Hangul Syllables
	0x30A0..=0x30FF, // Katakana
	0x3040..=0x309F, // Hiragana
	0x4E00..=0x9FFF, // CJK Unified Ideographs
	0x3400..=0x4DBF, // CJK Unified Ideographs Extension A
	0xF900..=0xFAFF, // CJK Compatibility Ideographs
	0x0980..=0x09FF, // Bengali
	0x0900..=0x097F, // Devanagari
	0x0400..=0x04FF, // Cyrillic
	0x0500..=0x052F, // Cyrillic Supplementary
	0x0370..=0x03FF, // Greek and Coptic
	0x1F00..=0x1FFF, // Greek Extended
	0x0E00..=0x0E7F, // Thai
	0x0600..=0x06FF, // Arabic
	0xFB50..=0xFDFF, // Arabic Presentation Forms-A
	0x0590..=0x05FF, // Hebrew
];
