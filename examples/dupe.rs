/// Find nouns with both singular and plural entries
use anyhow::Result;
use booky::word::{Lexicon, Word, WordClass};

fn main() -> Result<()> {
    let lex = Lexicon::builtin();
    for word in lex.iter() {
        if keep(&lex, word) {
            println!("{word:?}");
        }
    }
    Ok(())
}

fn keep(lex: &Lexicon, word: &Word) -> bool {
    if WordClass::Noun == word.word_class() {
        for w in lex.iter() {
            if WordClass::Noun == w.word_class() {
                if w != word {
                    for form in w.forms() {
                        if form == word.base() {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}
