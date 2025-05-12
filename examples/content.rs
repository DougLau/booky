// Basic HTML content parsing
use anyhow::Result;
use core::str;
use html_escape::decode_html_entities;
use std::io::{BufRead, stdin};

struct Content {
    /// read buffer
    buf: Vec<u8>,
    /// DOM element stack display bool
    stack: Vec<bool>,
}

fn main() -> Result<()> {
    let mut content = Content::new();
    content.parse(stdin().lock())?;
    Ok(())
}

const NON_CLOSING: &[&str] = &["!--", "img", "input", "link", "meta", "source"];

const NON_DISPLAYED: &[&str] = &[
    "a",
    "annotation",
    "figure",
    "footer",
    "form",
    "header",
    "label",
    "nav",
    "script",
    "semantics",
    "style",
];

impl Content {
    fn new() -> Self {
        Content {
            buf: Vec::with_capacity(4096),
            stack: Vec::new(),
        }
    }

    fn is_content_displayed(&self) -> bool {
        !self.stack.iter().any(|d| !d)
    }

    fn handle_content(&self) {
        if self.is_content_displayed() {
            if let Ok(text) = str::from_utf8(&self.buf) {
                let text = text.trim();
                if !text.is_empty() {
                    println!("{}", decode_html_entities(text));
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
                        let mut parts = text.trim().split_whitespace();
                        if let Some(elem) = parts.next() {
                            if NON_CLOSING.contains(&elem) {
                                continue;
                            }
                            if elem.starts_with('/') {
                                self.stack.pop();
                            } else {
                                let displayed = !NON_DISPLAYED.contains(&elem);
                                let hidden = parts.any(is_class_hidden);
                                self.stack.push(displayed && !hidden);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn is_class_hidden(part: &str) -> bool {
    part.starts_with("class") && {
        part.contains("catlinks")
            || part.contains("sidebar-list")
            || part.contains("infobox")
            || part.contains("vector-menu")
            || part.contains("references")
    }
}
