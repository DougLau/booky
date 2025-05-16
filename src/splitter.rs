use std::io::{self, BufRead, Bytes};

/// Character chunk types
#[derive(Clone, Debug, PartialEq)]
pub enum Chunk {
    /// Alphanumeric character or apostrophe text
    Text(char),
    /// Any non-`Text` displayable character
    Symbol(char),
    /// Discard character
    Discard,
}

/// Splitter for separating string chunks
///
/// All whitespace and control characters are discarded.
pub struct WordSplitter<R: BufRead> {
    /// Remaining bytes of underlying reader
    bytes: Bytes<R>,
    /// Current unicode UTF-8 code
    code: Vec<u8>,
}

impl<R> WordSplitter<R>
where
    R: BufRead,
{
    /// Create a new word splitter
    pub fn new(r: R) -> Self {
        WordSplitter {
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
                    if let Ok(c) = core::str::from_utf8(&self.code) {
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

impl<R> Iterator for WordSplitter<R>
where
    R: BufRead,
{
    type Item = Result<Chunk, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_char() {
            Some(Ok(c)) => Some(Ok(Chunk::from_char(c))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

impl Chunk {
    /// Determine chunk type from a single character
    fn from_char(c: char) -> Self {
        if c.is_whitespace() || c.is_control() || c == '\u{FEFF}' {
            // ZERO WIDTH NO-BREAK SPACE `U+FEFF` is sometimes used as a BOM
            Chunk::Discard
        } else if c.is_alphanumeric() || is_apostrophe(c) {
            Chunk::Text(c)
        } else {
            Chunk::Symbol(c)
        }
    }
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
