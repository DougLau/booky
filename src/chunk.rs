use std::io::{self, BufRead, Bytes};

/// Handler for parsing text chunks
pub trait ChunkHandler {
    /// Handle a text chunk
    fn text(&mut self, ch: &str);
    /// Handle a symbol chunk
    fn symbol(&mut self, ch: &str);
    /// Handle a boundary chunk
    fn boundary(&mut self, ch: &str);
}

/// Character chunk types
#[derive(Clone, Copy, Debug, PartialEq)]
enum Chunk {
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

/// Parse text into chunks
pub fn parse_text<R, H>(
    reader: R,
    handler: &mut H,
) -> Result<(), std::io::Error>
where
    R: BufRead,
    H: ChunkHandler,
{
    let mut chunk = String::new();
    for ch in CharSplitter::new(reader) {
        let c = ch?;
        match Chunk::from_char(c) {
            Chunk::Boundary => {
                handle_text(handler, &mut chunk);
                chunk.push(c);
                handler.boundary(&chunk);
                chunk.clear();
            }
            Chunk::Symbol => {
                if c == '-' {
                    // double dash means no more compound
                    if !chunk.is_empty() && !chunk.ends_with('-') {
                        chunk.push('-');
                        continue;
                    }
                }
                if c == '.' && is_dot_appendable(&chunk) {
                    chunk.push('.');
                    continue;
                }
                handle_text(handler, &mut chunk);
                chunk.push(c);
                handler.symbol(&chunk);
                chunk.clear();
            }
            Chunk::Text => match canonical_char(c) {
                Some(s) => chunk.push_str(s),
                None => chunk.push(c),
            },
        }
    }
    handle_text(handler, &mut chunk);
    Ok(())
}

/// Handle text chunk
fn handle_text<H>(handler: &mut H, chunk: &mut String)
where
    H: ChunkHandler,
{
    if !chunk.is_empty() {
        // this check doesn't work for abbreviations...
        if chunk.ends_with('.')
            && chunk.chars().filter(|c| *c == '.').count() == 1
        {
            chunk.pop();
            handler.text(chunk);
            chunk.clear();
            chunk.push('.');
            handler.symbol(chunk);
            chunk.clear();
        } else {
            handler.text(chunk);
        }
        chunk.clear();
    }
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
    } else if c == 'æ' {
        Some("ae")
    } else {
        None
    }
}
