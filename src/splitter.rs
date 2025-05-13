use std::io::{self, Bytes, Read};

/// Splitter for separating text into "chunks" and "symbols".
///
/// Chunks are strings of alphanumeric characters, hyphens, periods, or
/// 4 types of apostrophe.  Symbols are any other displayable characters.
/// All whitespace and control characters are stripped.
pub struct WordSplitter<R: Read> {
    /// Remaining bytes of underlying reader
    bytes: Bytes<R>,
    /// Current unicode UTF-8 code
    code: Vec<u8>,
    /// Next character
    next: Option<char>,
}

impl<R> WordSplitter<R>
where
    R: Read,
{
    /// Create a new word splitter
    pub fn new(r: R) -> Self {
        WordSplitter {
            bytes: r.bytes(),
            code: Vec::with_capacity(4),
            next: None,
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
        Some(Err(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8")))
    }
}

impl<R> Iterator for WordSplitter<R>
where
    R: Read,
{
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = String::new();
        if let Some(c) = self.next.take() {
            chunk.push(c);
        }
        while let Some(c) = self.next_char() {
            match c {
                Ok(next) => {
                    if should_discard(next) {
                        if !chunk.is_empty() {
                            return Some(Ok(chunk));
                        }
                    } else {
                        match chunk.chars().next() {
                            Some(c) => {
                                if is_chunk_char(c) && is_chunk_char(next) {
                                    chunk.push(next);
                                } else {
                                    self.next = Some(next);
                                    return Some(Ok(chunk));
                                }
                            }
                            None => {
                                chunk.push(next);
                            }
                        }
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
        if !chunk.is_empty() {
            Some(Ok(chunk))
        } else {
            None
        }
    }
}

/// Check if a character should be discarded
fn should_discard(c: char) -> bool {
    c.is_whitespace() || c == '\u{FEFF}' || c.is_control()
}

/// Check if a character is part of a chunk
fn is_chunk_char(c: char) -> bool {
    c.is_alphanumeric() || is_apostrophe(c) || c == '-' || c == '.'
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunks() {
        for (val, chunks) in [
            ("", [].as_slice()),
            ("a", ["a"].as_slice()),
            (" a ", ["a"].as_slice()),
            ("a b", ["a", "b"].as_slice()),
            ("a \t\n b", ["a", "b"].as_slice()),
            ("a.b", ["a.b"].as_slice()),
            ("a-b", ["a-b"].as_slice()),
            ("a,b", ["a", ",", "b"].as_slice()),
            ("a?b!", ["a", "?", "b", "!"].as_slice()),
            ("abc 123", ["abc", "123"].as_slice()),
            ("...", ["..."].as_slice()),
            ("!!!", ["!", "!", "!"].as_slice()),
        ] {
            assert_eq!(
                WordSplitter::new(val.as_bytes())
                    .map(|w| w.unwrap())
                    .collect::<Vec<_>>(),
                chunks
            );
        }
    }
}
