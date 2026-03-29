use crate::config::Tier;

pub struct Language {
    pub code: &'static str,
    pub name: &'static str,
}

#[cfg(feature = "fluid_audio")]
pub const PARAKEET_LANGUAGES: &[&str] = &[
    "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu", "it", "lv", "lt", "mt",
    "pl", "pt", "ro", "ru", "sk", "sl", "es", "sv", "uk",
];

pub fn is_supported_on_tier(code: &str, tier: &Tier) -> bool {
    match tier {
        Tier::Fast => {
            #[cfg(feature = "fluid_audio")]
            if crate::platform::is_apple_silicon() {
                return PARAKEET_LANGUAGES.contains(&code);
            }
            let _ = code;
            true
        }
        Tier::Standard | Tier::Accurate => true,
    }
}

pub const LANGUAGES: &[Language] = &[
    // Common (top 20)
    Language {
        code: "en",
        name: "English",
    },
    Language {
        code: "zh",
        name: "Chinese",
    },
    Language {
        code: "de",
        name: "German",
    },
    Language {
        code: "es",
        name: "Spanish",
    },
    Language {
        code: "ru",
        name: "Russian",
    },
    Language {
        code: "ko",
        name: "Korean",
    },
    Language {
        code: "fr",
        name: "French",
    },
    Language {
        code: "ja",
        name: "Japanese",
    },
    Language {
        code: "pt",
        name: "Portuguese",
    },
    Language {
        code: "tr",
        name: "Turkish",
    },
    Language {
        code: "pl",
        name: "Polish",
    },
    Language {
        code: "nl",
        name: "Dutch",
    },
    Language {
        code: "ar",
        name: "Arabic",
    },
    Language {
        code: "sv",
        name: "Swedish",
    },
    Language {
        code: "it",
        name: "Italian",
    },
    Language {
        code: "id",
        name: "Indonesian",
    },
    Language {
        code: "hi",
        name: "Hindi",
    },
    Language {
        code: "vi",
        name: "Vietnamese",
    },
    Language {
        code: "uk",
        name: "Ukrainian",
    },
    Language {
        code: "th",
        name: "Thai",
    },
    Language {
        code: "ur",
        name: "Urdu",
    },
    // Rest (alphabetical by name)
    Language {
        code: "af",
        name: "Afrikaans",
    },
    Language {
        code: "sq",
        name: "Albanian",
    },
    Language {
        code: "am",
        name: "Amharic",
    },
    Language {
        code: "hy",
        name: "Armenian",
    },
    Language {
        code: "as",
        name: "Assamese",
    },
    Language {
        code: "az",
        name: "Azerbaijani",
    },
    Language {
        code: "ba",
        name: "Bashkir",
    },
    Language {
        code: "eu",
        name: "Basque",
    },
    Language {
        code: "be",
        name: "Belarusian",
    },
    Language {
        code: "bn",
        name: "Bengali",
    },
    Language {
        code: "bs",
        name: "Bosnian",
    },
    Language {
        code: "br",
        name: "Breton",
    },
    Language {
        code: "bg",
        name: "Bulgarian",
    },
    Language {
        code: "yue",
        name: "Cantonese",
    },
    Language {
        code: "ca",
        name: "Catalan",
    },
    Language {
        code: "hr",
        name: "Croatian",
    },
    Language {
        code: "cs",
        name: "Czech",
    },
    Language {
        code: "da",
        name: "Danish",
    },
    Language {
        code: "et",
        name: "Estonian",
    },
    Language {
        code: "fo",
        name: "Faroese",
    },
    Language {
        code: "fi",
        name: "Finnish",
    },
    Language {
        code: "gl",
        name: "Galician",
    },
    Language {
        code: "ka",
        name: "Georgian",
    },
    Language {
        code: "el",
        name: "Greek",
    },
    Language {
        code: "gu",
        name: "Gujarati",
    },
    Language {
        code: "ht",
        name: "Haitian Creole",
    },
    Language {
        code: "ha",
        name: "Hausa",
    },
    Language {
        code: "haw",
        name: "Hawaiian",
    },
    Language {
        code: "he",
        name: "Hebrew",
    },
    Language {
        code: "hu",
        name: "Hungarian",
    },
    Language {
        code: "is",
        name: "Icelandic",
    },
    Language {
        code: "jw",
        name: "Javanese",
    },
    Language {
        code: "kn",
        name: "Kannada",
    },
    Language {
        code: "kk",
        name: "Kazakh",
    },
    Language {
        code: "km",
        name: "Khmer",
    },
    Language {
        code: "lo",
        name: "Lao",
    },
    Language {
        code: "la",
        name: "Latin",
    },
    Language {
        code: "lv",
        name: "Latvian",
    },
    Language {
        code: "ln",
        name: "Lingala",
    },
    Language {
        code: "lt",
        name: "Lithuanian",
    },
    Language {
        code: "lb",
        name: "Luxembourgish",
    },
    Language {
        code: "mk",
        name: "Macedonian",
    },
    Language {
        code: "mg",
        name: "Malagasy",
    },
    Language {
        code: "ms",
        name: "Malay",
    },
    Language {
        code: "ml",
        name: "Malayalam",
    },
    Language {
        code: "mt",
        name: "Maltese",
    },
    Language {
        code: "mi",
        name: "Maori",
    },
    Language {
        code: "mr",
        name: "Marathi",
    },
    Language {
        code: "mn",
        name: "Mongolian",
    },
    Language {
        code: "my",
        name: "Myanmar",
    },
    Language {
        code: "ne",
        name: "Nepali",
    },
    Language {
        code: "no",
        name: "Norwegian",
    },
    Language {
        code: "nn",
        name: "Nynorsk",
    },
    Language {
        code: "oc",
        name: "Occitan",
    },
    Language {
        code: "ps",
        name: "Pashto",
    },
    Language {
        code: "fa",
        name: "Persian",
    },
    Language {
        code: "pa",
        name: "Punjabi",
    },
    Language {
        code: "ro",
        name: "Romanian",
    },
    Language {
        code: "sa",
        name: "Sanskrit",
    },
    Language {
        code: "sr",
        name: "Serbian",
    },
    Language {
        code: "sn",
        name: "Shona",
    },
    Language {
        code: "sd",
        name: "Sindhi",
    },
    Language {
        code: "si",
        name: "Sinhala",
    },
    Language {
        code: "sk",
        name: "Slovak",
    },
    Language {
        code: "sl",
        name: "Slovenian",
    },
    Language {
        code: "so",
        name: "Somali",
    },
    Language {
        code: "su",
        name: "Sundanese",
    },
    Language {
        code: "sw",
        name: "Swahili",
    },
    Language {
        code: "tl",
        name: "Tagalog",
    },
    Language {
        code: "tg",
        name: "Tajik",
    },
    Language {
        code: "ta",
        name: "Tamil",
    },
    Language {
        code: "tt",
        name: "Tatar",
    },
    Language {
        code: "te",
        name: "Telugu",
    },
    Language {
        code: "bo",
        name: "Tibetan",
    },
    Language {
        code: "tk",
        name: "Turkmen",
    },
    Language {
        code: "uz",
        name: "Uzbek",
    },
    Language {
        code: "cy",
        name: "Welsh",
    },
    Language {
        code: "yi",
        name: "Yiddish",
    },
    Language {
        code: "yo",
        name: "Yoruba",
    },
];
