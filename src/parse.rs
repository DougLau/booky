use crate::contractions;
use crate::kind::Kind;
use crate::word::Lexicon;
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
    lex: Lexicon,
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

/// Make "canonical" English spelling of a character
fn canonical_char(c: char) -> Option<&'static str> {
    if is_apostrophe(c) {
        Some("’")
    } else {
        None
    }
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
    pub fn new(lex: Lexicon, reader: R) -> Self {
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
                Chunk::Text => match canonical_char(c) {
                    Some(s) => self.text.push_str(s),
                    None => self.text.push(c),
                },
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
            || !txt.contains(['-', '’'])
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
            self.push_word_splittable(ch);
            first = false;
        }
    }

    /// Push a splittable (contraction) word
    fn push_word_splittable(&mut self, word: &str) {
        if word.contains('’') && !self.lex.contains(word) {
            for word in contractions::split(word) {
                if !word.is_empty() {
                    self.push_word(Chunk::Text, String::from(word));
                }
            }
        } else if !word.is_empty() {
            self.push_word(Chunk::Text, String::from(word));
        }
    }

    /// Push one word
    fn push_word(&mut self, chunk: Chunk, word: String) {
        let kind = if self.lex.contains(&word) {
            Kind::Lexicon
        } else {
            Kind::from(&word[..])
        };
        self.chunks.push(Ok((chunk, word, kind)));
    }
}
