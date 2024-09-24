use crate::validator::{
	consists_of_supported_unicode_character_sets, contains_blocked_characters,
	is_reserved_canonical_handle,
};

#[test]
fn test_is_reserved_canonical_handle_happy_path() {
	let reserved_handles: Vec<&str> =
		vec!["admin", "everyone", "all", "mod", "moderator", "administrator", "here", "channel"];

	for handle in reserved_handles {
		assert!(is_reserved_canonical_handle(crate::convert_to_canonical(handle).as_str()));
	}
}

#[test]
fn test_is_reserved_canonical_handle_negative() {
	let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont"];
	for handle in handles {
		assert!(!is_reserved_canonical_handle(crate::convert_to_canonical(handle).as_str()));
	}
}

#[test]
fn test_contains_blocked_characters_happy_path() {
	let handles: Vec<&str> =
		vec!["@lbert", "coca:cola", "#freemont", "charles.darwin", "`String`", ":(){ :|:& };:/"];
	for handle in handles {
		assert!(contains_blocked_characters(handle));
	}
}

#[test]
fn test_contains_blocked_characters_negative() {
	let handles: Vec<&str> =
		vec!["albert", "coca_cola", "freemont", "charles-darwin", "Single Quote'd"];
	for handle in handles {
		assert!(!contains_blocked_characters(handle));
	}
}

// To validate new test cases, add a string/sentence in the new language, run the test
// A test of a sentence can reveal character ranges needed for language rendering.
// Unicode groups: https://www.unicodepedia.com/groups/  for character ranges
// If you don't know why a test is failing, decode the string here to check the range:
//     https://unicodedecode.com/
// Translations of "I can eat glass" from https://www.kermitproject.org/utf8.html
#[rustfmt::skip]
#[test]
fn test_consists_of_supported_unicode_character_sets_happy_path() {
	let strings_containing_characters_in_supported_unicode_character_sets = Vec::from([
		"John",                                                     // Basic Latin
		"Álvaro",                                                   // Latin-1 Supplement
		"가영",                                                     // Hangul Syllables
		"가나다",                                                   // Hangul Syllables
		"アキラ",                                                   // Katakana
		"あいこ",                                                   // Hiragana
		"李明",                                                     // CJK Unified Ideographs
		"严勇",                                                     // CJK Unified Ideographs
		"龍",                                                       // CJK Unified Ideographs
		"অমিত",                                                     // Bengali
		"आरव",                                                      // Devanagari
		"Александр",                                                // Cyrillic
		"Αλέξανδρος",                                               // Greek and Coptic
		"Ἀναξαγόρας",                                               // Greek Extended
		"ฉันกินกระจกได้แต่มันไม่ทำให้ฉันเจ็บ",                               // Thai
		"ابجدهوزحطيكلمنسعفصقرشتثخذضظغءعمر",                         // Arabic
		"דָּנִיֵּאלאבּבגּגדּדהווּוֹזחטי ִיכּךּכךלמםנןסעפּףּפףצץקרשׁשׂתּת",            // Hewbrew
		"AaĄąBbCcĆćDdEeĘęFfGgHhIiJjKkLlŁłMmNnŃńOoÓóRrSsŚśYyZzŹźŻż", // Polish
		"ÄäÖöÜüẞß",                                                 // German
		"AÁBCČDĎEÉĚFGHChIÍJKLMNŇOÓPQRŘSŠTŤUÚŮVWXYÝZŽaábcčdďeéěfghchiíjklmnňoópqrřsštťuúůvwxyýzž", // Czech
		"αιαιαιᾳειειηιῃοιοιυιυιωιῳαυαυᾹυᾱυευευηυηυουουωυωυγγγγγκγκγξγξγχγχμπμπντντΖζΤΖτζ", // Greek
		"ÅåÄäÖö",                                                   // Swedish
		"ÅåÄäÖöŠšŽž",                                               // Finnish
		"ÆæØøÅå",                                                   // Danish
		"ᏌᏃᏂ ᎠᏁᏴ",                                                 // Cherokee
		"Կրնամ",                                                   // Armenian
		"शक्नोम्यत्तुम्",                                                  // Devanagari
		"მინას",                                                    // Georgian
		"আমিকাঁচখেতেপারিতাতেআমারকোনোক্ষতিহয়না।",                           // Bengali
		"मीकाचखाऊशकतोमलातेदुखतनाही",                                     // Marathi
		"ನನಗೆಹಾನಿಆಗದೆ,ನಾನುಗಜನ್ನುತಿನಬಹುದು",                              // Kannada
		"मैंकाँचखासकतीहूँऔरमुझेउससेकोईचोटनहींपहुंचती",                            // Hindi
		"நான்கண்ணாடிசாப்பிடுவேன்,அதனால்எனக்குஒருகேடும்வராது",              // Tamil
		"నేనుగాజుతినగలనుమరియుఅలాచేసినానాకుఏమిఇబ్బందిలేదు",                // Telugu
		" මටවීදුරුකෑමටහැකියි.එයින්මටකිසිහානියක්සිදුනොවේ",                        // Sinhalese
		"میں کانچکھاسکتاہوںورمجھےتکلیفنہیںہوتی",                    // Urdu
		"شيشهخوړلېشمهغه ما نه خوږوي",                               // Pashto
		" .من می توانم بدونِ احساس درد شيشه بخورم",                  // Farsi / Persian(3)
		"أنا قادر على أكل الزجاج و هذا لا يؤلمني. ",                // Arabic
		" إِنا إِىَ تَونَر غِلَاشِ كُمَ إِن غَمَا لَافِىَا",                        // Hausa
		"က္ယ္ဝန္‌တော္‌၊က္ယ္ဝန္‌မ မ္ယက္‌စားနုိင္‌သည္‌။ ၎က္ရောင္‌့ ထိခုိက္‌မ္ဟု မရ္ဟိပာ။",            // Burmese (Unicode 4.0):
		"ကျွန်တော် ကျွန်မ မှန်စားနိုင်တယ်။ ၎င်းကြောင့် ထိခိုက်မှုမရှိပါ။",                // Burmese (Unicode 5.0):
		"Tôi có thể ăn thủy tinh mà không hại gì.",                 // Vietnamese (quốc ngữ)
		" ខ្ញុំអាចញុំកញ្ចក់បាន ដោយគ្មានបញ្ហារ ",                             // Khmer:
		"ຂອ້ຍກິນແກ້ວໄດ້ໂດຍທີ່ມັນບໍ່ໄດ້ເຮັດໃຫ້ຂອ້ຍເຈັບ",                           // Lao:
		"Би шил идэй чадна, надад хортой биш",                      // Mongolian (Cyrillic):
		"ᠪᠢ ᠰᠢᠯᠢ ᠢᠳᠡᠶᠦ ᠴᠢᠳᠠᠨᠠ ᠂ ᠨᠠᠳᠤᠷ ᠬᠣᠤᠷᠠᠳᠠᠢ ᠪᠢᠰᠢ",               // Mongolian (Classic) (5):
		"म काँच खान सक्छू र मलाई केहि नी हुन्‍न् ।",                        // Nepali:
		" ཤེལ་སྒོ་ཟ་ནས་ང་ན་གི་མ་རེད།",                                   // Tibetan:
		" 我能吞下玻璃而不伤身体",                                  // Chinese:
		" 我能吞下玻璃而不傷身體",                                  // Chinese (Traditional):
		"Góa ē-tàng chia̍h po-lê mā bē tio̍h-siong",                  // Taiwanese
		" 私はガラスを食べられますそれは私を傷つけません",          // Japanese:
		" 나는 유리를 먹을 수 있어요. 그래도 아프지 않아요",        // Korean:
		" ᐊᓕᒍᖅ ᓂᕆᔭᕌᖓᒃᑯ ᓱᕋᙱᑦᑐᓐᓇᖅᑐᖓ",                                 // Inuktitut
		" Tsésǫʼ yishą́ągo bííníshghah dóó doo shił neezgai da.",    // Navajo:
		" mi kakne le nu citka le blaci .iku'i le se go'i na xrani mi", // Lojban:
		" Ljœr ye caudran créneþ ý jor cẃran.",                     // Nórdicg:
		" Ég get etið gler án þess að meiða mig.",                  // Íslenska / Icelandic
		" Mogę jeść szkło, i mi nie szkodzi.",                      // Polish:
		" Pot să mănânc sticlă și ea nu mă rănește.",               // Romanian:
		" Я можу їсти шкло, й воно мені не пошкодить.",             // Ukrainian:
		" Կրնամ ապակի ուտել և ինծի անհանգիստ չըներ։",               // Armenian:
	]);

	for string in strings_containing_characters_in_supported_unicode_character_sets {
		assert!(consists_of_supported_unicode_character_sets(string));
	}
}

#[test]
fn test_consists_of_supported_unicode_character_sets_rejects_emojis() {
	// Constructing a string that with the smiling face emoji
	let string_containing_emojis = format_args!("John{}", '\u{1F600}').to_string();

	assert!(!consists_of_supported_unicode_character_sets(&string_containing_emojis));
}

// Will load `CONFUSABLES` with all the confusables at build time.
// See build.rs
include!(concat!(env!("OUT_DIR"), "/confusables.rs"));

#[test]
fn test_confusables_map_does_not_contain_keys_in_unsupported_character_sets() {
	for key in CONFUSABLES.keys() {
		assert!(consists_of_supported_unicode_character_sets(&key.to_string()));
	}
}
