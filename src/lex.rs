use crate::word::Word;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Static lexicon
static LEXICON: LazyLock<Lexicon> = LazyLock::new(make_builtin);

/// Make builtin lexicon
fn make_builtin() -> Lexicon {
    let mut lex = Lexicon::default();
    for (i, line) in include_str!("../res/english.csv").lines().enumerate() {
        match Word::try_from(line) {
            Ok(word) => lex.insert(word),
            Err(_) => eprintln!("Bad word on line {}: `{line}`", i + 1),
        }
    }
    lex
}

/// Get built-in lexicon
pub fn builtin() -> &'static Lexicon {
    &LEXICON
}

/// Lexicon of words
#[derive(Default, Clone)]
pub struct Lexicon {
    /// Words
    words: Vec<Word>,
    /// All word forms
    forms: HashMap<String, Vec<usize>>,
}

impl IntoIterator for Lexicon {
    type Item = Word;
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

    /// Insert a word into the lexicon
    pub fn insert(&mut self, word: Word) {
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

    /// Check if lexicon contains a word
    pub fn contains(&self, word: &str) -> bool {
        self.forms.contains_key(&word.to_lowercase())
    }

    /// Get all entries containing a word
    pub fn word_entries(&self, word: &str) -> Vec<&Word> {
        if let Some(indices) = self.forms.get(&word.to_lowercase()) {
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

    /// Get an iterator of all words
    pub fn iter(&self) -> impl Iterator<Item = &Word> {
        self.words.iter()
    }
}
