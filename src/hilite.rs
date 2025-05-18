use crate::kind::Kind;
use crate::lex;
use crate::parse::Parser;
use crate::word::WordClass;
use std::io::BufRead;
use yansi::{Paint, Style};

/// Hilite text from a reader
pub fn hilite_text<R>(reader: R) -> Result<(), std::io::Error>
where
    R: BufRead,
{
    for chunk in Parser::new(reader) {
        let (_chunk, text, kind) = chunk?;
        print!("{}", text.paint(style(kind, &text)));
    }
    println!();
    Ok(())
}

/// Get style to paint a chunk
fn style(kind: Kind, word: &str) -> Style {
    match kind {
        Kind::Lexicon => {
            let Some(wc) = word_class(word) else {
                return Style::new();
            };
            match wc {
                WordClass::Noun => Style::new().bright_blue().bold(),
                WordClass::Pronoun => Style::new().bright_blue().italic(),
                WordClass::Adjective => Style::new().bright_cyan().bold(),
                WordClass::Verb => Style::new().bright_green(),
                WordClass::Adverb => Style::new().green(),
                _ => Style::new().bright_white(),
            }
        }
        Kind::Foreign => Style::new().bright().bold().italic(),
        Kind::Ordinal | Kind::Roman | Kind::Number => {
            Style::new().bright_red().bold()
        }
        Kind::Acronym => Style::new().bold(),
        Kind::Proper => Style::new().bright().bold(),
        Kind::Symbol => Style::new().dim(),
        Kind::Unknown => Style::new().underline(),
    }
}

/// Determine word class
fn word_class(word: &str) -> Option<WordClass> {
    let mut ents = lex::builtin().word_entries(word);
    if ents.len() == 1 {
        let we = ents.pop().unwrap();
        Some(we.word_class())
    } else {
        // FIXME: match sentence structure to choose word class?
        None
    }
}
