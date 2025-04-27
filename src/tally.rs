use crate::word::Dict;
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;

/// Word contractions
enum Contraction {
    Full(&'static str, &'static str, &'static str),
    Suffix(&'static str, &'static str),
}

/// Word tally entry
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordEntry {
    /// Seen count
    seen: usize,
    /// Word
    word: String,
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

/// Check if a character should be trimmed at word start
fn is_trim_start(c: char) -> bool {
    !c.is_alphanumeric() && !is_apostrophe(c)
}

/// Check if a character should be trimmed at word end
fn is_trim_end(c: char) -> bool {
    !c.is_alphabetic() && !is_apostrophe(c)
}

/// Trim non-word characters from start and end of a word
fn trim_word(w: &str) -> &str {
    w.trim_start_matches(is_trim_start)
        .trim_end_matches(is_trim_end)
}

/// Check if a character is part of a word
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || is_apostrophe(c) || c == '-'
}

/// Check if a string is a valid word
fn is_word_valid(w: &str) -> bool {
    w.chars().all(is_word_char) && !w.is_empty()
}

/// Check if a character is a romal numeral
fn is_roman_numeral_char(c: char) -> bool {
    ['I', 'V', 'X', 'L', 'C', 'D', 'M'].contains(&c)
}

/// Check if a string is a romal numeral (not "I")
fn is_roman_numeral(w: &str) -> bool {
    w.chars().all(is_roman_numeral_char) && !w.is_empty() && w != "I"
}

impl fmt::Display for WordEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:5} {}", self.seen, self.word)?;
        Ok(())
    }
}

impl TryFrom<&str> for WordEntry {
    type Error = ();

    fn try_from(word: &str) -> Result<Self, Self::Error> {
        if is_word_valid(word) && !is_roman_numeral(word) {
            Ok(WordEntry::new(1, word))
        } else {
            Err(())
        }
    }
}

/// Make "canonical" English spelling of a word
fn canonical_spelling(word: &str) -> String {
    let word = trim_word(word);
    word.replace(is_apostrophe, "’").replace('æ', "ae")
}

impl WordEntry {
    /// Create a new word entry
    fn new(seen: usize, word: &str) -> Self {
        let word = word.to_string();
        WordEntry { seen, word }
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

    /// Get mutable word
    fn word_mut(&mut self) -> &mut String {
        &mut self.word
    }

    /// Check if a word is probably proper
    fn is_probably_proper(&self) -> bool {
        let mut chars = self.word.chars();
        match chars.next() {
            Some(c) if c.is_uppercase() => chars.all(|c| c.is_lowercase()),
            _ => false,
        }
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
    Contraction::Suffix("’d", "would"),
    Contraction::Suffix("’s", ""), // possessive
    Contraction::Suffix("’", ""),  // possessive
];

impl Contraction {
    /// Check if a word uses the contraction
    fn check(&self, word: &str) -> bool {
        match self {
            Contraction::Full(c, _, _) => word.eq_ignore_ascii_case(c),
            Contraction::Suffix(s, _) => word.ends_with(s),
        }
    }

    /// Expand the contraction
    fn expand<'a>(&self, word: &'a str) -> Vec<&'a str> {
        match self {
            Contraction::Full(_, a, b) => vec![a, b],
            Contraction::Suffix(s, ex) => match word.strip_suffix(s) {
                Some(base) => vec![base, ex],
                None => vec![word],
            },
        }
    }
}

/// Split contractions
fn split_contractions(word: &str) -> impl Iterator<Item = &str> {
    for con in CONTRACTIONS {
        if con.check(word) {
            return con.expand(word).into_iter();
        }
    }
    vec![word].into_iter()
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
                    *e.word_mut() = we.word;
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

    /// Take all "probably" proper nouns into a new tally
    pub fn take_proper(&mut self) -> Self {
        let mut other = WordTally::new();
        for we in self.words.values() {
            if we.is_probably_proper() {
                other.tally_word(we.word(), we.seen());
            }
        }
        for key in other.words.keys() {
            self.words.remove(&key[..]);
        }
        other
    }

    /// Get a Vec of word entries
    pub fn into_entries(self) -> Vec<WordEntry> {
        let mut entries: Vec<_> = self.words.into_values().collect();
        entries.sort();
        entries
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

    /// Remove single-letter words not in dictionary
    pub fn remove_single(&mut self, dict: &Dict) {
        self.words
            .retain(|_key, we| dict.contains(we.word()) || we.word().len() > 1);
    }
}
