use crate::word::Lexeme;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Static lexicon
static LEXICON: LazyLock<Lexicon> = LazyLock::new(make_builtin);

/// Make builtin lexicon
fn make_builtin() -> Lexicon {
    let mut lex = Lexicon::default();
    for (i, line) in include_str!("../res/english.csv").lines().enumerate() {
        match Lexeme::try_from(line) {
            Ok(word) => lex.insert(word),
            Err(_) => panic!("Bad word on line {}: `{line}`", i + 1),
        }
    }
    lex
}

/// Get built-in lexicon
pub fn builtin() -> &'static Lexicon {
    &LEXICON
}

/// Check if a character is an apostrophe
///
/// Unicode has several different apostrophes:
///  - ' `U+0027` (ASCII APOSTROPHE)
///  - ʼ `U+02BC` (MODIFIER LETTER APOSTROPHE) -- glottal stop
///  - ’ `U+2019` (RIGHT SINGLE QUOTATION MARK) -- recommended by Unicode!
///  - ＇ `U+FF07` (FULLWIDTH APOSTROPHE)
pub fn is_apostrophe(c: char) -> bool {
    c == '\u{0027}' || c == '\u{02BC}' || c == '\u{2019}' || c == '\u{FF07}'
}

/// Make word to check lexicon
pub fn make_word(word: &str) -> String {
    let mut w = String::with_capacity(word.len());
    for c in word.chars() {
        if is_apostrophe(c) {
            w.push('\'');
        } else {
            for cl in c.to_lowercase() {
                w.push(cl);
            }
        }
    }
    w
}

/// Lexicon of words
#[derive(Default, Clone)]
pub struct Lexicon {
    /// All lexemes
    words: Vec<Lexeme>,
    /// All word forms
    forms: HashMap<String, Vec<usize>>,
}

impl IntoIterator for Lexicon {
    type Item = Lexeme;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.words.sort();
        self.words.into_iter()
    }
}

impl Lexicon {
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Lexicon::default()
    }

    /// Insert a lexeme (word) into the lexicon
    pub fn insert(&mut self, word: Lexeme) {
        for form in word.forms() {
            self.insert_form(form);
        }
        self.words.push(word);
    }

    /// Insert a word form
    fn insert_form(&mut self, word: &str) {
        let n = self.words.len();
        if let Some(nums) = self.forms.get_mut(word) {
            nums.push(n);
        } else {
            let nums = vec![n];
            self.forms.insert(word.to_lowercase(), nums);
        }
    }

    /// Check if lexicon contains a word form
    pub fn contains(&self, word: &str) -> bool {
        self.forms.contains_key(&make_word(word))
    }

    /// Get all lexeme entries containing a word form
    pub fn word_entries(&self, word: &str) -> Vec<&Lexeme> {
        if let Some(indices) = self.forms.get(&make_word(word)) {
            let mut entries = Vec::with_capacity(indices.len());
            for i in indices {
                entries.push(&self.words[*i]);
            }
            return entries;
        }
        vec![]
    }

    /// Get an iterator of all word forms (lowercase)
    pub fn forms(&self) -> impl Iterator<Item = &String> {
        self.forms.keys()
    }

    /// Get an iterator of all lexemes (words)
    pub fn iter(&self) -> impl Iterator<Item = &Lexeme> {
        self.words.iter()
    }
}
