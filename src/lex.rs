use crate::word::Word;
use std::collections::HashMap;

/// Lexicon of words
#[derive(Default)]
pub struct Lexicon {
    /// Words
    words: Vec<Word>,
    /// All word forms
    forms: HashMap<String, Vec<usize>>,
}

impl Lexicon {
    /// Create a new empty lexicon
    pub fn new() -> Self {
        Lexicon::default()
    }

    /// Get built-in lexicon
    pub fn builtin() -> Self {
        let mut lex = Lexicon::default();
        for (i, line) in include_str!("../res/english.csv").lines().enumerate()
        {
            match Word::try_from(line) {
                Ok(word) => lex.insert(word),
                Err(_) => eprintln!("Bad word on line {}: `{line}`", i + 1),
            }
        }
        lex
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

    /// Sort the words
    pub fn sort(&mut self) {
        self.words.sort();
    }

    /// Check if lexicon contains a word
    pub fn contains(&self, word: &str) -> bool {
        self.forms.contains_key(&word.to_lowercase())
    }

    /// Get an iterator of words
    pub fn iter(&self) -> impl Iterator<Item = &Word> {
        self.words.iter()
    }

    /// Get an iterator of all word forms (lowercase)
    pub fn forms(&self) -> impl Iterator<Item = &String> {
        self.forms.keys()
    }
}
