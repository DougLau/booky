use anyhow::Result;
use booky::word::{Dict, Word, WordClass};

fn main() -> Result<()> {
    let dict = Dict::builtin();
    for word in dict.iter() {
        if keep(&dict, word) {
            println!("{word:?}");
        }
    }
    Ok(())
}

fn keep(dict: &Dict, word: &Word) -> bool {
    if let Some(WordClass::Noun) = word.word_class() {
        for w in dict.iter() {
            if let Some(WordClass::Noun) = w.word_class() {
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
