/// Find nouns with both singular and plural entries
use anyhow::Result;
use booky::lex::{self, Lexicon};
use booky::word::{Lexeme, WordClass};

fn main() -> Result<()> {
    let lex = lex::builtin();
    for word in lex.iter() {
        if !keep(&lex, word) {
            println!("{word:?}");
        }
    }
    Ok(())
}

fn keep(lex: &Lexicon, word: &Lexeme) -> bool {
    if WordClass::Noun == word.word_class() {
        for w in lex.iter() {
            if WordClass::Noun == w.word_class() {
                if w != word {
                    for form in w.forms() {
                        if form == word.lemma() {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}
