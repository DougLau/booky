use std::collections::HashMap;
use std::fmt;

/// Word attributes
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum WordAttr {
    /// `a`: Auxiliary verb (e.g. "cannot")
    Auxiliary,
    /// `c`: Count noun (e.g. "chair")
    Countable,
    /// `u`: Uncountable noun (mass, e.g. "furniture")
    Uncountable,
    /// `g` Group (collective noun)
    Group,
    /// `n`: Proper (name) noun
    Proper,
    /// `p`: Plurale Tantum (e.g. "pants" or "scissors")
    PluraleTantum,
    /// `i` Intransitive verb or preposition
    Intransitive,
    /// `t` Transitive verb or preposition
    Transitive,
    /// `j` Conjunctive preposition
    Conjunctive,
    /// `v` "Deverbal" prepositions (e.g. "excluding")
    Deverbal,
}

/// Word class
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum WordClass {
    /// `A`: Adjective
    Adjective,
    /// `Av`: Adverb
    Adverb,
    /// `C`: Conjunction
    Conjunction,
    /// `D`: Determiner
    Determiner,
    /// `I`: Interjection
    Interjection,
    /// `N`: Noun
    #[default]
    Noun,
    /// `P`: Preposition
    Preposition,
    /// `Pn`: Pronoun
    Pronoun,
    /// `V`: Verb
    Verb,
}

/// Word
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Word {
    /// Base word form
    base: String,
    /// Word class
    word_class: Option<WordClass>,
    /// Attributes
    attr: String,
    /// Irregular forms
    irregular_forms: Vec<String>,
    /// All forms
    forms: Vec<String>,
}

/// Dictionary of words
#[derive(Default)]
pub struct Dict {
    /// Words
    words: Vec<Word>,
    /// All word forms
    forms: HashMap<String, Vec<usize>>,
}

impl TryFrom<char> for WordAttr {
    type Error = ();

    fn try_from(val: char) -> Result<Self, Self::Error> {
        match val {
            'a' => Ok(Self::Auxiliary),
            'c' => Ok(Self::Countable),
            'u' => Ok(Self::Uncountable),
            'g' => Ok(Self::Group),
            'n' => Ok(Self::Proper),
            'p' => Ok(Self::PluraleTantum),
            'i' => Ok(Self::Intransitive),
            't' => Ok(Self::Transitive),
            'j' => Ok(Self::Conjunctive),
            'v' => Ok(Self::Deverbal),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for WordClass {
    type Error = ();

    fn try_from(cl: &str) -> Result<Self, Self::Error> {
        match cl {
            "N" => Ok(WordClass::Noun),
            "V" => Ok(WordClass::Verb),
            "A" => Ok(WordClass::Adjective),
            "Av" => Ok(WordClass::Adverb),
            "P" => Ok(WordClass::Preposition),
            "Pn" => Ok(WordClass::Pronoun),
            "C" => Ok(WordClass::Conjunction),
            "D" => Ok(WordClass::Determiner),
            "I" => Ok(WordClass::Interjection),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WordClass {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let wc = match self {
            WordClass::Noun => "N",
            WordClass::Verb => "V",
            WordClass::Adjective => "A",
            WordClass::Adverb => "Av",
            WordClass::Preposition => "P",
            WordClass::Pronoun => "Pn",
            WordClass::Conjunction => "C",
            WordClass::Determiner => "D",
            WordClass::Interjection => "I",
        };
        write!(fmt, "{wc}")
    }
}

impl TryFrom<&str> for Word {
    type Error = ();

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let mut vals = line.split(',');
        let base = vals.next().filter(|b| !b.is_empty()).ok_or(())?;
        let (base, cla) = base.split_once(':').unwrap_or((base, ""));
        let base = base.to_string();
        let (wc, a) = cla.split_once('.').unwrap_or((cla, ""));
        let word_class = WordClass::try_from(wc).ok();
        let attr = a.to_string();
        let mut irregular_forms = Vec::new();
        for form in vals {
            irregular_forms.push(form.replace("_", &base));
        }
        let mut forms = Vec::new();
        forms.push(base.clone());
        for form in &irregular_forms {
            if *form != base {
                forms.push(form.clone());
            }
        }
        let mut word = Word {
            base,
            word_class,
            attr,
            irregular_forms,
            forms,
        };
        if word.irregular_forms.is_empty() {
            word.build_regular_forms();
        }
        Ok(word)
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.base)?;
        if let Some(wc) = self.word_class {
            write!(fmt, ":{wc}")?;
            if !self.attr.is_empty() {
                write!(fmt, ".{}", self.attr)?;
            }
            for form in &self.irregular_forms {
                match form.strip_prefix(&self.base) {
                    Some(suffix) => write!(fmt, ",_{suffix}")?,
                    None => write!(fmt, ",{form}")?,
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Word {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.base)?;
        if let Some(wc) = self.word_class {
            write!(fmt, ":{wc}")?;
            if !self.attr.is_empty() {
                write!(fmt, ".{}", self.attr)?;
            }
        }
        Ok(())
    }
}

impl Word {
    /// Get base word as a string slice
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Get the word class
    pub fn word_class(&self) -> Option<WordClass> {
        self.word_class
    }

    /// Get irregular forms
    pub fn irregular_forms(&self) -> &[String] {
        &self.irregular_forms[..]
    }

    /// Get all forms
    pub fn forms(&self) -> &[String] {
        &self.forms[..]
    }

    /// Check if a word (noun) has plural form
    fn has_plural(&self) -> bool {
        for a in self.attr.chars() {
            if let Ok(WordAttr::Uncountable | WordAttr::PluraleTantum) =
                WordAttr::try_from(a)
            {
                return false;
            }
        }
        true
    }

    /// Build regular word forms
    fn build_regular_forms(&mut self) {
        match self.word_class {
            Some(WordClass::Noun) => {
                if self.has_plural() {
                    self.forms.push(noun_plural(&self.base));
                }
            }
            Some(WordClass::Verb) => {
                self.forms.push(verb_present(&self.base));
                self.forms.push(verb_present_participle(&self.base));
                self.forms.push(verb_past(&self.base));
            }
            Some(WordClass::Adjective) => {
                self.forms.push(adjective_comparative(&self.base));
                self.forms.push(adjective_superlative(&self.base));
            }
            _ => (),
        }
    }
}

/// Make a regular plural noun from the singular form
fn noun_plural(base: &str) -> String {
    if base.ends_with("s")
        || base.ends_with("sh")
        || base.ends_with("ch")
        || base.ends_with("x")
        || base.ends_with("z")
    {
        format!("{base}es")
    } else if ends_in_y(base) {
        let base = base.trim_end_matches('y');
        format!("{base}ies")
    } else if ends_in_o(base) {
        format!("{base}es")
    } else {
        format!("{base}s")
    }
}

/// Check if a character is a vowel
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Check if a word ends with a consonant which should repeat
fn consonant_end_repeat(s: &str) -> Option<char> {
    // ideally, we would determine whether the final syllable is stressed
    let mut suffix = (' ', ' ', ' ');
    for c in s.chars() {
        if suffix.2 == 'q' && c == 'u' {
            // "qu" does not contain a vowel
            continue;
        }
        suffix = (suffix.1, suffix.2, c);
    }
    // check for exception suffixes
    if let 'w' | 'x' = suffix.2 {
        return None;
    }
    if let ('e', 'd') | ('e', 'n') | ('e', 'r') | ('o', 'n') =
        (suffix.1, suffix.2)
    {
        return None;
    }
    if !is_vowel(suffix.0) && is_vowel(suffix.1) && !is_vowel(suffix.2) {
        Some(suffix.2)
    } else {
        None
    }
}

/// Make a regular present verb from the base form
fn verb_present(base: &str) -> String {
    if base.ends_with("s") || base.ends_with("z") {
        match consonant_end_repeat(base) {
            Some(end) => format!("{base}{end}es"),
            None => format!("{base}es"),
        }
    } else if base.ends_with("sh") {
        format!("{base}es")
    } else if ends_in_y(base) {
        let base = base.trim_end_matches('y');
        format!("{base}ies")
    } else {
        format!("{base}s")
    }
}

/// Check if a base word ends in `o` (with no other vowel)
fn ends_in_o(base: &str) -> bool {
    base.ends_with("o")
        && !(base.ends_with("ao")
            || base.ends_with("eo")
            || base.ends_with("io")
            || base.ends_with("oo")
            || base.ends_with("uo"))
}

/// Check if a base word ends in `y` (with no other vowel)
fn ends_in_y(base: &str) -> bool {
    base.ends_with("y")
        && !(base.ends_with("ay")
            || base.ends_with("ey")
            || base.ends_with("iy")
            || base.ends_with("oy")
            || base.ends_with("uy")
            || base.ends_with("yy"))
}

/// Check if a base word ends in `e`
fn ends_in_e(base: &str) -> bool {
    base.ends_with("e")
        && !(base.ends_with("ae")
            || base.ends_with("ee")
            || base.ends_with("ie")
            || base.ends_with("oe")
            || base.ends_with("ye"))
}

/// Make a regular present participle verb from the base form
fn verb_present_participle(base: &str) -> String {
    if let Some(end) = consonant_end_repeat(base) {
        return format!("{base}{end}ing");
    }
    if ends_in_e(base) {
        let base = base.trim_end_matches('e');
        format!("{base}ing")
    } else {
        format!("{base}ing")
    }
}

/// Make a regular past verb from the base form
fn verb_past(base: &str) -> String {
    if let Some(end) = consonant_end_repeat(base) {
        return format!("{base}{end}ed");
    }
    if base.ends_with("e") {
        format!("{base}d")
    } else if ends_in_y(base) {
        let base = base.trim_end_matches('y');
        format!("{base}ied")
    } else {
        format!("{base}ed")
    }
}

/// Make a regular comparative adjective from the base form
fn adjective_comparative(base: &str) -> String {
    if base.ends_with("e") {
        return format!("{base}r");
    } else if ends_in_y(base) {
        let base = base.trim_end_matches('y');
        return format!("{base}ier");
    }
    match consonant_end_repeat(base) {
        Some(end) => format!("{base}{end}er"),
        None => format!("{base}er"),
    }
}

/// Make a regular superlative adjective from the base form
fn adjective_superlative(base: &str) -> String {
    if base.ends_with("e") {
        return format!("{base}st");
    } else if ends_in_y(base) {
        let base = base.trim_end_matches('y');
        return format!("{base}iest");
    }
    match consonant_end_repeat(base) {
        Some(end) => format!("{base}{end}est"),
        None => format!("{base}est"),
    }
}

impl Dict {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Dict::default()
    }

    /// Get built-in dictionary
    pub fn builtin() -> Self {
        let mut dict = Dict::default();
        for (i, line) in include_str!("../res/english.csv").lines().enumerate()
        {
            match Word::try_from(line) {
                Ok(word) => dict.insert(word),
                Err(_) => eprintln!("Bad word on line {}: `{line}`", i + 1),
            }
        }
        dict
    }

    /// Insert a word into the dictionary
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

    /// Check if dictionary contains a word
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
