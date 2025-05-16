use crate::splitter::{Chunk, WordSplitter};
use crate::word::Lexicon;
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

/// Word kind
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Dictionary
    Dictionary,
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

/// Word tally entry
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordEntry {
    /// Seen count
    seen: usize,
    /// Word
    word: String,
    /// Kind grouping
    kind: Kind,
}

/// Word tally list
pub struct WordTally {
    /// Lexicon
    lex: Lexicon,
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

impl Kind {
    /// Get all word kinds
    pub fn all() -> &'static [Self] {
        use Kind::*;
        &[
            Dictionary, Foreign, Ordinal, Roman, Number, Acronym, Proper,
            Symbol, Unknown,
        ]
    }

    /// Get code
    pub fn code(self) -> char {
        use Kind::*;
        match self {
            Dictionary => 'd',
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
    let len = word.chars().count();
    let upper = word.chars().filter(|c| c.is_uppercase()).count();
    let dots = word.chars().filter(|c| *c == '.').count();
    // must have no dots or half dots
    len >= 2 && upper + dots == len && (dots == 0 || dots * 2 == len)
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
        let kind = self.kind().code();
        write!(fmt, "{:5} {} ", self.seen.bright().yellow(), kind.yellow())?;
        if let Some(c) = self.word.chars().next() {
            if c.is_control() || c == '\u{FEFF}' {
                return write!(fmt, "{}", c.escape_unicode());
            }
        }
        write!(fmt, "{}", self.word)
    }
}

/// Make "canonical" English spelling of a character
fn canonical_char(c: char) -> Option<&'static str> {
    if is_apostrophe(c) {
        Some("’")
    } else if c == 'æ' {
        Some("ae")
    } else {
        None
    }
}

impl WordEntry {
    /// Create a new word entry
    fn new(seen: usize, word: &str, kind: Kind) -> Self {
        let word = word.to_string();
        WordEntry { seen, word, kind }
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

    /// Get kind grouping
    pub fn kind(&self) -> Kind {
        self.kind
    }
}

/// Count the number of uppercase characters in a word
fn count_uppercase(word: &str) -> usize {
    word.chars().filter(|c| c.is_uppercase()).count()
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
    Contraction::Full("m’dear", "my", "dear"),
    Contraction::Full("m’lady", "my", "lady"),
    Contraction::Full("m’lord", "my", "lord"),
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
    pub fn new(lex: Lexicon) -> Self {
        WordTally {
            lex,
            words: HashMap::new(),
        }
    }

    /// Parse text from a reader
    pub fn parse_text<R>(&mut self, reader: R) -> Result<(), std::io::Error>
    where
        R: BufRead,
    {
        let mut compound = String::new();
        for chunk in WordSplitter::new(reader) {
            match chunk? {
                Chunk::Discard => {
                    self.tally_compound(&compound, 1);
                    String::clear(&mut compound);
                }
                Chunk::Symbol(c) => {
                    if c == '-' {
                        // double dash means no more compound
                        if !compound.is_empty() && !compound.ends_with('-') {
                            compound.push('-');
                            continue;
                        }
                    }
                    if c == '.' {
                        compound.push('.');
                        if is_acronym(&compound) {
                            continue;
                        } else {
                            compound.pop();
                        }
                    }
                    self.tally_compound(&compound, 1);
                    String::clear(&mut compound);
                    self.tally_word(&String::from(c), 1);
                }
                Chunk::Text(c) => match canonical_char(c) {
                    Some(s) => compound.push_str(s),
                    None => compound.push(c),
                },
            }
        }
        self.tally_compound(&compound, 1);
        Ok(())
    }

    /// Tally a compound cluster
    fn tally_compound(&mut self, compound: &str, count: usize) {
        if self.lex.contains(compound) {
            self.tally_word(compound, count);
            return;
        }
        // not in lexicon; split it up
        let mut first = false;
        for chunk in compound.split('-') {
            if !first {
                self.tally_word("-", count);
            }
            self.tally_word(chunk, count);
            first = false;
        }
    }

    /// Tally a word
    fn tally_word(&mut self, word: &str, count: usize) {
        if word.is_empty() {
            return;
        }
        let kind = if self.lex.contains(word) {
            Kind::Dictionary
        } else {
            Kind::from(word)
        };
        let we = WordEntry::new(1, word, kind);
        let key = word.to_lowercase();
        match self.words.get_mut(&key) {
            Some(e) => {
                // use variant with fewest uppercase characters
                if count_uppercase(we.word()) < count_uppercase(e.word()) {
                    e.word = we.word;
                    e.kind = we.kind;
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

    /// Count the words of a given kind
    pub fn count_kind(&self, kind: Kind) -> usize {
        self.words
            .iter()
            .filter(|(_k, we)| we.kind() == kind)
            .count()
    }

    /// Get a Vec of word entries
    pub fn into_entries(self) -> Vec<WordEntry> {
        let mut entries: Vec<_> = self.words.into_values().collect();
        entries.sort();
        entries
    }

    /// Split contractions (with apostrophe) not in lexicon
    pub fn split_unknown_contractions(&mut self) {
        let contractions: Vec<_> = self
            .words
            .iter()
            .filter(|(_k, we)| {
                !self.lex.contains(we.word()) && we.word().contains('’')
            })
            .map(|(key, _we)| key.clone())
            .collect();
        for key in contractions {
            if let Some(we) = self.words.remove(&key) {
                let con = we.word();
                for word in split_contractions(con) {
                    self.tally_word(word, we.seen());
                }
            }
        }
    }
}
