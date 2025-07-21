// We need the ALLOWED_UNICODE_CHARACTER_RANGES for the build as well as in the source, so here it is.

use core::ops::RangeInclusive;

#[cfg(test)]
#[allow(dead_code)]
pub fn build_allowed_char_ranges() -> Vec<RangeInclusive<u16>> {
	let mut new_allowed: Vec<RangeInclusive<u16>> = Vec::new();
	let mut last: RangeInclusive<u16> = RangeInclusive::new(0u16, 0u16);
	// assumes the list is sorted!
	for allowed in ALLOWED_UNICODE_CHARACTER_RANGES {
		let last_start = last.start();
		let last_end = last.end();
		let allowed_start = allowed.start();
		let allowed_end = allowed.end();
		if *allowed_start == *last_end + 1u16 {
			println!(
				"joining {last_start:#X}..{last_end:#X} with {allowed_start:#X}..#{allowed_end:#X}"
			);
			last = RangeInclusive::new(*last.start(), *allowed.end());
		} else {
			println!("adding {last_start:#X}..{last_end:#4X}");
			if *last_end > 0u16 {
				new_allowed.push(last.clone());
			}
			last = allowed.clone()
		}
	}
	new_allowed
}

/// Characters that are allowed.
/// This is generated using test_build_allowed_char_ranges
#[rustfmt::skip]
pub const ALLOWED_UNICODE_CHARACTER_RANGES: [RangeInclusive<u16>; 54] = [
0x0020..=0x007A,
0x0080..=0x024F,
0x02B0..=0x04FF,
0x0531..=0x058A,
0x0591..=0x05F4,
0x0600..=0x07B1,
0x07C0..=0x07FA,
0x0900..=0x097F,
0x0981..=0x09FB,
0x0A01..=0x0A75,
0x0A81..=0x0AF1,
0x0B01..=0x0B77,
0x0B82..=0x0BFA,
0x0C01..=0x0C7F,
0x0C82..=0x0CF2,
0x0D02..=0x0D7F,
0x0D82..=0x0DF4,
0x0E01..=0x0E5B,
0x0E81..=0x0EDD,
0x0F00..=0x0FDA,
0x1000..=0x10FC,
0x1100..=0x137C,
0x1380..=0x1399,
0x13A0..=0x13F4,
0x1400..=0x167F,
0x1700..=0x1714,
0x1720..=0x1736,
0x1740..=0x1753,
0x1760..=0x1773,
0x1780..=0x17F9,
0x1800..=0x18AA,
0x18B0..=0x18F5,
0x1900..=0x1974,
0x1980..=0x1AAD,
0x1B00..=0x1B7C,
0x1B80..=0x1BB9,
0x1BC0..=0x1C7F,
0x1E00..=0x1FFF,
0x200C..=0x200D,
0x2C80..=0x2CFF,
0x2D30..=0x2D7F,
0x3040..=0x30FF,
0x3400..=0x4DBF,
0x4E00..=0x9FFF,
0xA500..=0xA62B,
0xA880..=0xA8D9,
0xA8E0..=0xA8FB,
0xA900..=0xA95F,
0xA980..=0xA9DF,
0xAA00..=0xAA7B,
0xAA80..=0xAADF,
0xABC0..=0xABF9,
0xAC00..=0xD7AF,
0xF900..=0xFAFF,
];

// Keep this to show what languages are supported and to generate a new compact
// list whenever the list is updated.
// pub const ALLOWED_UNICODE_CHARACTER_RANGES: [RangeInclusive<u16>; 75] = [
//     0x0020..=0x007A, // BasicLatin
//     0x0080..=0x00FF, // Latin-1 Supplement
//     0x0100..=0x017F, // Latin Extended-A
//     0x0180..=0x024F,   // Latin Extended-B
//     0x02B0..=0x02FF, // Spacing Modifier Letters
//     0x0300..=0x036F, // Combining diacritical marks
//     0x0370..=0x03FF, // Greek and Coptic
//     0x0400..=0x04FF, // Cyrillic
//     0x0531..=0x058A, // Armenian
//     0x0591..=0x05F4, // Hebrew
//     0x0600..=0x06FF, // Arabic
//     0x0700..=0x074F, // Syriac
//     0x0750..=0x077F, // ArabicSupplement
//     0x0780..=0x07B1, // Thaana
//     0x07C0..=0x07FA, // N'Ko
//     0x0900..=0x097F, // Devanagari
//     0x0981..=0x09FB, // Bengali
//     0x0A01..=0x0A75, // Gurmukhi
//     0x0A81..=0x0AF1, // Gujarati
//     0x0B01..=0x0B77, // Oriya
//     0x0B82..=0x0BFA, // Tamil
//     0x0C01..=0x0C7F, // Telugu
//     0x0C82..=0x0CF2, // Kannada
//     0x0D02..=0x0D7F, // Malayalam
//     0x0D82..=0x0DF4, // Sinhala
//     0x0E01..=0x0E5B, // Thai
//     0x0E81..=0x0EDD, // Lao
//     0x0F00..=0x0FDA, // Tibetan
//     0x1000..=0x109F, // Myanmar
//     0x10A0..=0x10FC, // Georgian
//     0x1100..=0x11FF, // HangulJamo
//     0x1200..=0x137C, // Ethiopic
//     0x1380..=0x1399, // EthiopicSupplement
//     0x13A0..=0x13F4, // Cherokee
//     0x1400..=0x167F, // UnifiedCanadianAboriginalSyllabics
//     0x1700..=0x1714, // Tagalog
//     0x1720..=0x1736, // Hanunoo
//     0x1740..=0x1753, // Buhid
//     0x1760..=0x1773, // Tagbanwa
//     0x1780..=0x17F9, // Khmer
//     0x1800..=0x18AA, // Mongolian
//     0x18B0..=0x18F5, // Unified Canadian Aboriginal Syllabics Extended
//     0x1900..=0x194F, // Limbu
//     0x1950..=0x1974, // Tai Le
//     0x1980..=0x19DF, // New Tai Le
//     0x19E0..=0x19FF, // Khmer Symbols
//     0x1A00..=0x1A1F, // Buginese
//     0x1A20..=0x1AAD, // Tai Tham
//     0x1B00..=0x1B7C, // Balinese
//     0x1B80..=0x1BB9, // Sundanese
//     0x1BC0..=0x1BFF, // Batak
//     0x1C00..=0x1C4F, // Lepcha
//     0x1C50..=0x1C7F, // Ol Chiki
//     0x1E00..=0x1EFF, // Latin Extended Additional
//     0x1F00..=0x1FFF, // Greek Extended
//     0x200C..=0x200D, // General punctuation Limited to the Zero-width Joiners
//     0x2C80..=0x2CFF, // Coptic
//     0x2D30..=0x2D7F, // Tifinagh
//     0x3040..=0x309F, // Hiragana
//     0x30A0..=0x30FF, // Katakana
//     0x3400..=0x4DBF, // CJK Unified Ideographs Extension A
//     0x4E00..=0x9FFF, // CJK Unified Ideographs
//     0xA500..=0xA62B, // Vai
//     0xA880..=0xA8D9, // Saurashtra
//     0xA8E0..=0xA8FB, // Devanagari Extended
//     0xA900..=0xA92F, // Kayah Li
//     0xA930..=0xA95F, // Rejang
//     0xA980..=0xA9DF, // Javanese
//     0xAA00..=0xAA5F, // Cham
//     0xAA60..=0xAA7B, // Myanmar Extended-A
//     0xAA80..=0xAADF, // Tai Viet
//     0xABC0..=0xABF9, // Meetei Mayek
//     0xAC00..=0xD7AF, // Hangul Syllables
//     0xF900..=0xFAFF, // CJK Compatibility Ideographs
//     0xFB50..=0xFDFF, // Arabic Presentation Forms-A
// ];

// You can comment out the current one and uncomment the original, specific one
// for all the languages supported.
#[test]
#[ignore = "use only to regenerate compacted ALLOWED_UNICODE_CHARACTER_RANGES"]
fn test_build_allowed_char_ranges() {
	let res = build_allowed_char_ranges();
	assert_eq!(res.len(), 54usize);
	for range in res {
		let start = range.start();
		let end = range.end();
		println!("{start:#4X}..={end:#4X},")
	}
}
