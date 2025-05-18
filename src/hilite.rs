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
            let mut ents = lex::builtin().word_entries(word);
            if ents.len() != 1 {
                return Style::new();
            }
            let word = ents.pop().unwrap();
            match word.word_class() {
                WordClass::Noun => Style::new().bright_red(),
                WordClass::Pronoun => Style::new().red(),
                WordClass::Verb => Style::new().bright_green(),
                WordClass::Adverb => Style::new().bright_cyan(),
                _ => Style::new().bright_white(),
            }
        }
        Kind::Foreign => Style::new().bright().italic(),
        Kind::Ordinal | Kind::Roman | Kind::Number => {
            Style::new().bright_blue()
        }
        Kind::Proper => Style::new().bold(),
        Kind::Symbol => Style::new().dim(),
        Kind::Unknown => Style::new().underline(),
        _ => Style::new(),
    }
}
