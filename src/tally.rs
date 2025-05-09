use crate::word::Dict;
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;
use yansi::Paint;

/// Uppercase roman numerals
const ROMAN_UPPER: &str = "IVXLCDM";

/// Lowercase roman numerals
const ROMAN_LOWER: &str = "ivxlcdm";

/// Word contractions
enum Contraction {
    Full(&'static str, &'static str, &'static str),
    Prefix(&'static str, &'static str),
    Suffix(&'static str, &'static str),
}

/// Word category
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// Dictionary
    Dictionary,
    /// Ordinal number
    Ordinal,
    /// Roman numeral
    Roman,
    /// Number (may include letters)
    Number,
    /// Acronym / Initialism
    Acronym,
    /// Foreign (non-English)
    Foreign,
    /// Proper noun (name)
    Proper,
    /// Single letter
    Letter,
    /// Unknown / Other
    Unknown,
}

/// Word tally entry
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordEntry {
    /// Seen count
    seen: usize,
    /// Word
    word: String,
    /// Category
    cat: Category,
}

/// Word tally list
#[derive(Default)]
pub struct WordTally {
    /// Words in list
    words: HashMap<String, WordEntry>,
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

/// Check if a character is part of a word
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || is_apostrophe(c) || c == '-' || c == '.'
}

/// Check if a string is a valid word
fn is_word_valid(w: &str) -> bool {
    w.chars().all(is_word_char) && !w.is_empty()
}

impl Category {
    /// Get all categories
    pub fn all() -> &'static [Self] {
        use Category::*;
        &[
            Dictionary, Ordinal, Roman, Number, Acronym, Foreign, Proper,
            Letter, Unknown,
        ]
    }

    /// Get category code
    pub fn code(self) -> char {
        use Category::*;
        match self {
            Dictionary => 'd',
            Ordinal => 'o',
            Roman => 'r',
            Number => 'n',
            Acronym => 'a',
            Foreign => 'f',
            Proper => 'p',
            Letter => 'l',
            Unknown => 'u',
        }
    }
}

impl From<&str> for Category {
    fn from(word: &str) -> Self {
        if is_foreign(word) {
            Category::Foreign
        } else if is_ordinal_number(word) {
            Category::Ordinal
        } else if is_roman_numeral(word) {
            Category::Roman
        } else if is_number(word) {
            Category::Number
        } else if is_acronym(word) {
            Category::Acronym
        } else if is_probably_proper(word) {
            Category::Proper
        } else if word.len() == 1 {
            Category::Letter
        } else {
            Category::Unknown
        }
    }
}

/// Ordinal suffixes
const ORD_SUFFIXES: &[&str] =
    &["1st", "1ST", "2nd", "2ND", "3rd", "3RD", "th", "TH"];

/// Check if a string is an ordinal number
fn is_ordinal_number(w: &str) -> bool {
    for suf in ORD_SUFFIXES {
        if let Some(p) = w.strip_suffix(suf) {
            return !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
        }
    }
    false
}

/// Check if a string is a romal numeral
fn is_roman_numeral(word: &str) -> bool {
    let word = word.trim_end_matches('.');
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
    word.len() >= 2 && word.chars().all(|c| c.is_uppercase() || c == '.')
}

/// Check if a word is foreign (not English)
fn is_foreign(word: &str) -> bool {
    word.chars().any(|c| {
        !c.is_ascii_alphanumeric() && c != '-' && c != '.' && c != '\u{2019}'
    })
}

/// Check if a word is probably proper
fn is_probably_proper(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => chars.any(|c| c.is_lowercase()),
        _ => false,
    }
}

impl fmt::Display for WordEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let cat = self.category().code();
        write!(
            fmt,
            "{:5} {} {}",
            self.seen.bright().yellow(),
            cat.yellow(),
            self.word
        )?;
        Ok(())
    }
}

impl TryFrom<&str> for WordEntry {
    type Error = ();

    fn try_from(word: &str) -> Result<Self, Self::Error> {
        if is_word_valid(word) {
            Ok(WordEntry::new(1, word))
        } else {
            Err(())
        }
    }
}

/// Make "canonical" English spelling of a word
fn canonical_spelling(word: &str) -> String {
    let word = word
        .trim_start_matches(is_trim_start)
        .trim_end_matches(is_trim_end);
    word.replace(is_apostrophe, "’").replace('æ', "ae")
}

/// Check if a character should be trimmed at start of a word
fn is_trim_start(c: char) -> bool {
    c == '-' || c == '.' || !is_word_char(c)
}

/// Check if a character should be trimmed at end of a word
fn is_trim_end(c: char) -> bool {
    c == '-' || !is_word_char(c)
}

impl WordEntry {
    /// Create a new word entry
    fn new(seen: usize, word: &str) -> Self {
        let cat = Category::from(word);
        let word = word.to_string();
        WordEntry { seen, word, cat }
    }

    /// Get seen count
    pub fn seen(&self) -> usize {
        self.seen
    }

    /// Get mutable seen count
    fn seen_mut(&mut self) -> &mut usize {
        &mut self.seen
    }

    /// Get word
    pub fn word(&self) -> &str {
        &self.word
    }

    /// Guess word category
    pub fn category(&self) -> Category {
        self.cat
    }
}

/// Count the number of uppercase characters in a word
fn count_uppercase(word: &str) -> usize {
    word.chars().filter(|c| c.is_uppercase()).count()
}

/// Check if a word is a compound
fn is_compound(com: &str) -> bool {
    if com.contains('-') {
        for word in com.split('-') {
            if WordEntry::try_from(word).is_err() {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// Some contractions
const CONTRACTIONS: &[Contraction] = &[
    Contraction::Full("ain’t", "am", "not"),
    Contraction::Full("can’t", "can", "not"),
    Contraction::Full("shan’t", "shall", "not"),
    Contraction::Full("won’t", "will", "not"),
    Contraction::Suffix("n’t", "not"),
    Contraction::Suffix("’ve", "have"),
    Contraction::Suffix("’ll", "will"),
    Contraction::Full("I’m", "I", "am"),
    Contraction::Suffix("’re", "are"),
    Contraction::Full("he’s", "he", "is"),
    Contraction::Full("it’s", "it", "is"),
    Contraction::Full("she’s", "she", "is"),
    Contraction::Full("that’s", "that", "is"),
    Contraction::Full("there’s", "there", "is"),
    Contraction::Full("what’s", "what", "is"),
    Contraction::Full("who’s", "who", "is"),
    Contraction::Full("’tis", "it", "is"),
    Contraction::Full("’twas", "it", "was"),
    Contraction::Full("’twill", "it", "will"),
    Contraction::Suffix("’d", "would"),
    Contraction::Suffix("’s", ""), // possessive
    Contraction::Suffix("’", ""),  // possessive
    Contraction::Prefix("’", ""),  // weird quote
];

impl Contraction {
    /// Check if a word uses the contraction
    fn check(&self, word: &str) -> bool {
        match self {
            Contraction::Full(c, _, _) => word.eq_ignore_ascii_case(c),
            Contraction::Prefix(p, _) => word.starts_with(p),
            Contraction::Suffix(s, _) => word.ends_with(s),
        }
    }

    /// Expand the contraction
    fn expand<'a>(&self, word: &'a str) -> Vec<&'a str> {
        match self {
            Contraction::Full(_, a, b) => vec![a, b],
            Contraction::Prefix(p, ex) => match word.strip_prefix(p) {
                Some(base) => vec![base, ex],
                None => vec![word],
            },
            Contraction::Suffix(s, ex) => match word.strip_suffix(s) {
                Some(base) => vec![base, ex],
                None => vec![word],
            },
        }
    }
}

/// Split contractions
fn split_contractions(word: &str) -> Vec<&str> {
    let mut words = vec![word];
    let mut ex = Vec::with_capacity(2);
    while let Some(word) = words.pop() {
        let mut expanded = split_contraction(word);
        if expanded.is_empty() {
            ex.push(word);
        } else {
            words.append(&mut expanded);
        }
    }
    ex
}

/// Split one contraction
fn split_contraction(word: &str) -> Vec<&str> {
    for con in CONTRACTIONS {
        if con.check(word) {
            return con.expand(word);
        }
    }
    vec![]
}

impl WordTally {
    /// Create a new word tally
    pub fn new() -> Self {
        WordTally::default()
    }

    /// Parse text from a reader
    pub fn parse_text<R>(&mut self, reader: R) -> Result<(), std::io::Error>
    where
        R: BufRead,
    {
        for line in reader.lines() {
            for cluster in line?.split_whitespace() {
                // Don't allow double-hyphen in words
                for clump in cluster.split("--") {
                    self.tally_word(&canonical_spelling(clump), 1);
                }
            }
        }
        Ok(())
    }

    /// Tally a word
    fn tally_word(&mut self, word: &str, count: usize) {
        let Ok(we) = WordEntry::try_from(word) else {
            return;
        };
        let key = we.word().to_lowercase();
        match self.words.get_mut(&key) {
            Some(e) => {
                // use variant with fewest uppercase characters
                if count_uppercase(we.word()) < count_uppercase(e.word()) {
                    e.word = we.word;
                    e.cat = we.cat;
                }
                *e.seen_mut() += count;
            }
            None => {
                let mut we = we;
                *we.seen_mut() = count;
                self.words.insert(key, we);
            }
        }
    }

    /// Get the number of words
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Check if word tally is empty
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Count the words of a given category
    pub fn cat_count(&self, cat: Category) -> usize {
        self.words
            .iter()
            .filter(|(_k, we)| we.category() == cat)
            .count()
    }

    /// Get a Vec of word entries
    pub fn into_entries(self) -> Vec<WordEntry> {
        let mut entries: Vec<_> = self.words.into_values().collect();
        entries.sort();
        entries
    }

    /// Trim periods from end of words in dictionary
    pub fn trim_periods(&mut self, dict: &Dict) {
        let words: Vec<_> = self
            .words
            .iter()
            .filter(|(_k, we)| {
                !dict.contains(we.word()) && we.word().ends_with('.')
            })
            .map(|(key, _we)| key.clone())
            .collect();
        for key in words {
            if let Some(we) = self.words.get(&key) {
                let word = we.word().trim_end_matches('.');
                if dict.contains(word)
                    || is_roman_numeral(word)
                    || we.cat == Category::Proper
                {
                    let we = self.words.remove(&key).unwrap();
                    let word = we.word().trim_end_matches('.');
                    self.tally_word(word, we.seen());
                }
            }
        }
    }

    /// Split compound words (with hyphen) not in dictionary
    pub fn split_unknown_compounds(&mut self, dict: &Dict) {
        let compounds: Vec<_> = self
            .words
            .iter()
            .filter(|(_k, we)| {
                !dict.contains(we.word()) && is_compound(we.word())
            })
            .map(|(key, _we)| key.clone())
            .collect();
        for key in compounds {
            if let Some(we) = self.words.remove(&key) {
                for word in we.word().split('-') {
                    self.tally_word(word, we.seen());
                }
            }
        }
    }

    /// Split contractions (with apostrophe) not in dictionary
    pub fn split_unknown_contractions(&mut self, dict: &Dict) {
        let contractions: Vec<_> = self
            .words
            .iter()
            .filter(|(_k, we)| {
                !dict.contains(we.word()) && we.word().contains('’')
            })
            .map(|(key, _we)| key.clone())
            .collect();
        for key in contractions {
            if let Some(we) = self.words.remove(&key) {
                for word in split_contractions(we.word()) {
                    self.tally_word(word, we.seen());
                }
            }
        }
    }

    /// Check for word entries in dictionary
    pub fn check_dict(&mut self, dict: &Dict) {
        for (_key, we) in self.words.iter_mut() {
            if dict.contains(we.word()) {
                we.cat = Category::Dictionary;
            }
        }
    }
}
