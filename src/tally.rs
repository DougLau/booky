use crate::chunk::{ChunkHandler, parse_text};
use crate::contractions;
use crate::kind::Kind;
use crate::word::Lexicon;
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
pub struct WordTally {
    /// Lexicon
    lex: Lexicon,
    /// Words in list
    words: HashMap<String, WordEntry>,
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

impl ChunkHandler for &mut WordTally {
    fn text(&mut self, ch: &str) {
        self.tally_chunk(ch);
    }
    fn symbol(&mut self, ch: &str) {
        self.tally_chunk(ch);
    }
    fn discard(&mut self, _ch: &str) {
        // ignore discarded chunks
    }
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
        let mut h = self;
        parse_text(reader, &mut h)?;
        Ok(())
    }

    /// Tally a chunk
    fn tally_chunk(&mut self, chunk: &str) {
        if chunk.chars().count() < 2 || self.lex.contains(chunk) {
            self.tally_word(chunk, 1);
            return;
        }
        // not in lexicon; split up compound on hyphens
        let mut first = true;
        for chunk in chunk.split('-') {
            if !first {
                self.tally_word("-", 1);
            }
            self.tally_word(chunk, 1);
            first = false;
        }
    }

    /// Tally a word
    fn tally_word(&mut self, word: &str, count: usize) {
        if word.is_empty() {
            return;
        }
        let kind = if self.lex.contains(word) {
            Kind::Lexicon
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
                for word in contractions::split(con) {
                    self.tally_word(word, we.seen());
                }
            }
        }
    }
}
