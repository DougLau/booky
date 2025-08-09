use crate::kind::Kind;
use crate::lex::make_word;
use crate::parse::{Chunk, Parser};
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;
use yansi::Paint;

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
#[derive(Default)]
pub struct WordTally {
    /// Words in list
    words: HashMap<String, WordEntry>,
}

impl fmt::Display for WordEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let kind = self.kind().code();
        write!(fmt, "{:5} {} ", self.seen.bright().yellow(), kind.yellow())?;
        if let Some(c) = self.word.chars().next()
            && (c.is_control() || c == '\u{FEFF}')
        {
            return write!(fmt, "{}", c.escape_unicode());
        }
        write!(fmt, "{}", self.word)
    }
}

impl WordEntry {
    /// Create a new word entry
    fn new(seen: usize, word: String, kind: Kind) -> Self {
        WordEntry { seen, word, kind }
    }

    /// Get seen count
    pub fn seen(&self) -> usize {
        self.seen
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

impl WordTally {
    /// Create a new word tally
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse text from a reader
    pub fn parse_text<R>(&mut self, reader: R) -> Result<(), std::io::Error>
    where
        R: BufRead,
    {
        for chunk in Parser::new(reader) {
            let (chunk, text, kind) = chunk?;
            if chunk != Chunk::Boundary {
                self.tally_word(text, kind);
            }
        }
        Ok(())
    }

    /// Tally a word
    fn tally_word(&mut self, word: String, kind: Kind) {
        let key = make_word(&word);
        let we = WordEntry::new(1, word, kind);
        match self.words.get_mut(&key) {
            Some(e) => {
                // use variant with fewest uppercase characters
                if count_uppercase(we.word()) < count_uppercase(e.word()) {
                    e.word = we.word;
                    e.kind = we.kind;
                }
                e.seen += 1;
            }
            None => {
                let mut we = we;
                we.seen = 1;
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
}
