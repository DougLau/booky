use crate::contractions;
use crate::kind::Kind;
use crate::lex::{self, Lexicon};
use std::io::{self, BufRead, Bytes};

/// Character chunk types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chunk {
    /// Alphanumeric character or apostrophe text
    Text,
    /// Any non-`Text` displayable character
    Symbol,
    /// Word boundary character (whitespace, control, etc.)
    Boundary,
}

/// Splitter for separating text into characters
struct CharSplitter<R: BufRead> {
    /// Remaining bytes of underlying reader
    bytes: Bytes<R>,
    /// Current unicode UTF-8 code
    code: Vec<u8>,
}

/// Text parser
pub struct Parser<R: BufRead> {
    /// Word lexicon
    lex: &'static Lexicon,
    /// Text character splitter
    splitter: CharSplitter<R>,
    /// Current text chunk
    text: String,
    /// Processed chunks
    chunks: Vec<Result<(Chunk, String, Kind), io::Error>>,
}

impl<R> CharSplitter<R>
where
    R: BufRead,
{
    /// Create a new char splitter
    fn new(r: R) -> Self {
        CharSplitter {
            bytes: r.bytes(),
            code: Vec::with_capacity(4),
        }
    }

    /// Read the next character
    fn next_char(&mut self) -> Option<Result<char, io::Error>> {
        self.code.clear();
        for _i in 0..4 {
            match self.bytes.next() {
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(b)) => {
                    self.code.push(b);
                    if let Ok(c) = str::from_utf8(&self.code) {
                        if let Some(c) = c.chars().next() {
                            return Some(Ok(c));
                        }
                    }
                }
                None => {
                    if self.code.is_empty() {
                        return None;
                    } else {
                        break;
                    }
                }
            }
        }
        Some(Err(io::Error::other("Invalid UTF-8")))
    }
}

impl<R> Iterator for CharSplitter<R>
where
    R: BufRead,
{
    type Item = Result<char, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_char()
    }
}

impl Chunk {
    /// Determine chunk type from a single character
    fn from_char(c: char) -> Self {
        if is_boundary(c) {
            Chunk::Boundary
        } else if c.is_alphanumeric() || is_apostrophe(c) {
            Chunk::Text
        } else {
            Chunk::Symbol
        }
    }
}

/// Check if a character is a word "boundary" (non-Symbol)
fn is_boundary(c: char) -> bool {
    // ZERO WIDTH SPACE `U+200B` is a non-whitespace "space" (WTF?!)
    // ZERO WIDTH NO-BREAK SPACE `U+FEFF` is sometimes used as a BOM
    c.is_whitespace() || c.is_control() || c == '\u{200B}' || c == '\u{FEFF}'
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

/// Check if a dot is appendable
fn is_dot_appendable(word: &str) -> bool {
    word.chars().count() > 0
        && word.chars().all(|c| c.is_uppercase() || c == '.')
        && !word.ends_with('.')
}

impl<R> Iterator for Parser<R>
where
    R: BufRead,
{
    type Item = Result<(Chunk, String, Kind), io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunks.is_empty() {
            self.read_chunk();
        }
        if !self.chunks.is_empty() {
            Some(self.chunks.remove(0))
        } else {
            None
        }
    }
}

impl<R> Parser<R>
where
    R: BufRead,
{
    /// Create a new parser
    pub fn new(reader: R) -> Self {
        let lex = lex::builtin();
        let splitter = CharSplitter::new(reader);
        let chunks = Vec::new();
        let text = String::new();
        Parser {
            lex,
            splitter,
            text,
            chunks,
        }
    }

    /// Read next chunk
    fn read_chunk(&mut self) {
        while let Some(ch) = self.splitter.next() {
            if let Err(e) = ch {
                self.chunks.push(Err(e));
                return;
            }
            let c = ch.unwrap();
            match Chunk::from_char(c) {
                Chunk::Boundary => {
                    self.push_text();
                    self.push_boundary(c);
                    return;
                }
                Chunk::Symbol => {
                    if c == '-' {
                        // double dash means no more compound
                        if !self.text.is_empty() && !self.text.ends_with('-') {
                            self.text.push('-');
                            continue;
                        }
                    }
                    if c == '.' && is_dot_appendable(&self.text) {
                        self.text.push('.');
                        continue;
                    }
                    self.push_text();
                    self.push_symbol(c);
                    return;
                }
                Chunk::Text => self.text.push(c),
            }
        }
        self.push_text();
    }

    /// Push text chunk
    fn push_text(&mut self) {
        let mut text = std::mem::take(&mut self.text);
        if !text.is_empty() {
            // this check doesn't work for abbreviations...
            if text.ends_with('.')
                && text.chars().count() > 2
                && text.chars().filter(|c| *c == '.').count() == 1
            {
                text.pop();
                self.push_chunk(Chunk::Text, text);
                self.push_symbol('.');
            } else {
                self.push_chunk(Chunk::Text, text);
            }
        }
    }

    /// Push symbol chunk
    fn push_symbol(&mut self, c: char) {
        self.push_chunk(Chunk::Symbol, String::from(c));
    }

    /// Push boundary chunk
    fn push_boundary(&mut self, c: char) {
        self.push_chunk(Chunk::Boundary, String::from(c));
    }

    /// Push one chunk
    fn push_chunk(&mut self, chunk: Chunk, txt: String) {
        if txt.chars().count() == 1
            || self.lex.contains(&txt)
            || !txt.chars().any(is_splittable)
        {
            self.push_word(chunk, txt);
            return;
        }
        // not in lexicon; split up compound on hyphens
        let mut first = true;
        for ch in txt.split('-') {
            if !first {
                self.push_word(Chunk::Symbol, String::from('-'));
            }
            self.push_word_check_contraction(ch);
            first = false;
        }
    }

    /// Push a word (possible contraction)
    fn push_word_check_contraction(&mut self, word: &str) {
        if !word.is_empty() {
            let kind = self.contraction_kind(word);
            self.chunks
                .push(Ok((Chunk::Text, String::from(word), kind)));
        }
    }

    /// Check contraction kind
    fn contraction_kind(&self, word: &str) -> Kind {
        if self.lex.contains(word) {
            return Kind::Lexicon;
        }
        if word.chars().any(is_apostrophe) {
            let mut kinds = Vec::new();
            for w in contractions::split(word) {
                if !w.is_empty() {
                    let k = self.word_kind(w);
                    if k == Kind::Unknown {
                        return Kind::Unknown;
                    }
                    kinds.push(k);
                }
            }
            kinds.pop().unwrap_or(Kind::Unknown)
        } else {
            Kind::from(word)
        }
    }

    /// Get word kind
    fn word_kind(&self, word: &str) -> Kind {
        if self.lex.contains(word) {
            Kind::Lexicon
        } else {
            Kind::from(word)
        }
    }

    /// Push one word
    fn push_word(&mut self, chunk: Chunk, word: String) {
        let kind = self.word_kind(&word);
        self.chunks.push(Ok((chunk, word, kind)));
    }
}

/// Check if a character is splittable
fn is_splittable(c: char) -> bool {
    c == '-' || is_apostrophe(c)
}
