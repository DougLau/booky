use crate::kind::Kind;
use crate::parse::Parser;
use crate::lex::Lexicon;
use std::io::BufRead;
use yansi::Paint;

/// Hilite text from a reader
pub fn hilite_text<R>(lex: Lexicon, reader: R) -> Result<(), std::io::Error>
where
    R: BufRead,
{
    for chunk in Parser::new(lex, reader) {
        let (_chunk, text, kind) = chunk?;
        if kind == Kind::Unknown {
            print!("{}", text.underline());
        } else {
            print!("{text}");
        }
    }
    println!();
    Ok(())
}
