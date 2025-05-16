/// Word kind
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// In Lexicon
    Lexicon,
    /// Foreign (non-English)
    Foreign,
    /// Ordinal number
    Ordinal,
    /// Roman numeral
    Roman,
    /// Number (may include letters)
    Number,
    /// Acronym / Initialism
    Acronym,
    /// Proper noun (name)
    Proper,
    /// Symbol or letter (punctuation, etc.)
    Symbol,
    /// Unknown / Other
    Unknown,
}

impl Kind {
    /// Get all word kinds
    pub fn all() -> &'static [Self] {
        use Kind::*;
        &[
            Lexicon, Foreign, Ordinal, Roman, Number, Acronym, Proper, Symbol,
            Unknown,
        ]
    }

    /// Get code
    pub fn code(self) -> char {
        use Kind::*;
        match self {
            Lexicon => 'l',
            Foreign => 'f',
            Ordinal => 'o',
            Roman => 'r',
            Number => 'n',
            Acronym => 'a',
            Proper => 'p',
            Symbol => 's',
            Unknown => 'u',
        }
    }
}

impl From<&str> for Kind {
    fn from(word: &str) -> Self {
        if is_foreign(word) {
            Kind::Foreign
        } else if is_ordinal_number(word) {
            Kind::Ordinal
        } else if is_roman_numeral(word) {
            Kind::Roman
        } else if is_number(word) {
            Kind::Number
        } else if is_acronym(word) {
            Kind::Acronym
        } else if is_probably_proper(word) {
            Kind::Proper
        } else if word.chars().count() == 1 {
            Kind::Symbol
        } else {
            Kind::Unknown
        }
    }
}

/// Check if a word is foreign (not English)
fn is_foreign(word: &str) -> bool {
    word.chars()
        .any(|c| c.is_alphabetic() && !c.is_ascii() && !is_apostrophe(c))
}

/// Check if a character is an apostrophe
///
/// Unicode has several different apostrophes:
///  - ' `U+0027` (ASCII APOSTROPHE)
///  - ʼ `U+02BC` (MODIFIER LETTER APOSTROPHE)
///  - ’ `U+2019` (RIGHT SINGLE QUOTATION MARK) -- recommended by Unicode!
///  - ＇ `U+FF07` (FULLWIDTH APOSTROPHE)
fn is_apostrophe(c: char) -> bool {
    c == '\u{0027}' || c == '\u{02BC}' || c == '\u{2019}' || c == '\u{FF07}'
}

/// Ordinal suffixes
const ORD_SUFFIXES: &[&str] =
    &["1st", "1ST", "2nd", "2ND", "3rd", "3RD", "th", "TH"];

/// Check if a string is an ordinal number
fn is_ordinal_number(w: &str) -> bool {
    if w.chars().count() >= 3 {
        for suf in ORD_SUFFIXES {
            if let Some(p) = w.strip_suffix(suf) {
                return p.chars().all(|c| c.is_ascii_digit());
            }
        }
    }
    false
}

/// Uppercase roman numerals
const ROMAN_UPPER: &str = "IVXLCDM";

/// Lowercase roman numerals
const ROMAN_LOWER: &str = "ivxlcdm";

/// Check if a string is a romal numeral
fn is_roman_numeral(word: &str) -> bool {
    !word.is_empty()
        && (word.chars().all(|c| ROMAN_UPPER.contains(c))
            || word.chars().all(|c| ROMAN_LOWER.contains(c)))
}

/// Check if a word contains a number
fn is_number(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_digit())
}

/// Check if a word is an acronym / initialism
fn is_acronym(word: &str) -> bool {
    word.chars().count() >= 2
        && word.chars().all(|c| c.is_uppercase() || c == '.')
}

/// Check if a word is probably proper
fn is_probably_proper(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => chars.any(|c| c.is_lowercase()),
        _ => false,
    }
}
