// Basic HTML content parsing
use anyhow::Result;
use core::str;
use std::io::{BufRead, stdin};

struct Content {
    buf: Vec<u8>,
    stack: Vec<String>,
}

fn main() -> Result<()> {
    let mut content = Content::new();
    content.parse(stdin().lock())?;
    Ok(())
}

const NON_CLOSING: &[&str] = &["!--", "img", "input", "link", "meta", "source"];

const NON_DISPLAYED: &[&str] = &[
    "figure", "footer", "form", "header", "label", "nav", "script", "style",
];

impl Content {
    fn new() -> Self {
        Content {
            buf: Vec::with_capacity(4096),
            stack: Vec::new(),
        }
    }

    fn is_content_displayed(&self) -> bool {
        !self.stack.iter().any(|e| NON_DISPLAYED.contains(&&e[..]))
    }

    fn handle_content(&self) {
        if self.is_content_displayed() {
            if let Ok(text) = str::from_utf8(&self.buf) {
                let text = text.trim();
                if !text.is_empty() {
                    println!("{text}");
                }
            }
        }
    }

    fn parse<R>(&mut self, mut reader: R) -> Result<()>
    where
        R: BufRead,
    {
        loop {
            self.buf.clear();
            if reader.read_until(b'<', &mut self.buf)? == 0 {
                break;
            }
            if let Some(b'<') = self.buf.pop() {
                self.handle_content();
                self.buf.clear();
                if reader.read_until(b'>', &mut self.buf)? == 0 {
                    break;
                }
                if let Some(b'>') = self.buf.pop() {
                    if let Ok(text) = str::from_utf8(&self.buf) {
                        if let Some(elem) =
                            text.trim().split_whitespace().next()
                        {
                            if NON_CLOSING.contains(&elem) {
                                continue;
                            }
                            if elem.starts_with('/') {
                                self.stack.pop();
                            } else {
                                self.stack.push(elem.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
