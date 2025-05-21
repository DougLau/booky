use std::fmt;

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

/// Word attributes
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum WordAttr {
    /// `s`: Singulare Tantum (e.g. "dust" or "information")
    SingulareTantum,
    /// `p`: Plurale Tantum (e.g. "pants" or "scissors")
    PluraleTantum,
    /// `n`: Proper (name) noun
    Proper,
    /// `a`: Auxiliary verb (e.g. "cannot")
    Auxiliary,
    /// `i` Intransitive verb or preposition
    Intransitive,
    /// `t` Transitive verb or preposition
    Transitive,
}

/// Word Lexeme
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Lexeme {
    /// Lemma word form
    lemma: String,
    /// Word class
    word_class: WordClass,
    /// Attributes
    attr: String,
    /// Irregular forms
    irregular_forms: Vec<String>,
    /// All forms
    forms: Vec<String>,
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

impl TryFrom<char> for WordAttr {
    type Error = ();

    fn try_from(val: char) -> Result<Self, Self::Error> {
        match val {
            's' => Ok(Self::SingulareTantum),
            'p' => Ok(Self::PluraleTantum),
            'n' => Ok(Self::Proper),
            'a' => Ok(Self::Auxiliary),
            'i' => Ok(Self::Intransitive),
            't' => Ok(Self::Transitive),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Lexeme {
    type Error = ();

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let mut vals = line.split(',');
        let lemma = vals.next().filter(|v| !v.is_empty()).ok_or(())?;
        let (lemma, cla) = lemma.split_once(':').ok_or(())?;
        let lemma = lemma.to_string();
        let (wc, a) = cla.split_once('.').unwrap_or((cla, ""));
        let word_class = WordClass::try_from(wc)?;
        let attr = a.to_string();
        let mut irregular_forms = Vec::new();
        for form in vals {
            irregular_forms.push(form.replace("_", &lemma));
        }
        let mut forms = Vec::new();
        forms.push(lemma.clone());
        for form in &irregular_forms {
            if *form != lemma {
                forms.push(form.clone());
            }
        }
        let mut word = Lexeme {
            lemma,
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

impl fmt::Debug for Lexeme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.lemma, self.word_class)?;
        if !self.attr.is_empty() {
            write!(fmt, ".{}", self.attr)?;
        }
        for form in &self.irregular_forms {
            match form.strip_prefix(&self.lemma) {
                Some(suffix) => write!(fmt, ",_{suffix}")?,
                None => write!(fmt, ",{form}")?,
            }
        }
        Ok(())
    }
}

impl fmt::Display for Lexeme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.lemma, self.word_class)?;
        if !self.attr.is_empty() {
            write!(fmt, ".{}", self.attr)?;
        }
        Ok(())
    }
}

impl Lexeme {
    /// Get lemma as a string slice
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// Get the word class
    pub fn word_class(&self) -> WordClass {
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

    /// Count syllables in lemma form (poorly)
    fn count_syllables(&self) -> usize {
        let mut lemma = self.lemma();
        if ends_in_e(lemma) {
            lemma = lemma.trim_end_matches('e');
        }
        let mut syllables = 0;
        let mut letter = None;
        for c in lemma.chars() {
            if is_vowel(c) && !is_vowel(letter.unwrap_or(' ')) {
                syllables += 1;
            }
            letter = Some(c);
        }
        syllables
    }

    /// Check if a word (noun) has plural form
    fn has_plural(&self) -> bool {
        for a in self.attr.chars() {
            if let Ok(WordAttr::SingulareTantum | WordAttr::PluraleTantum) =
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
            WordClass::Adjective if self.count_syllables() < 4 => {
                self.forms.push(adjective_comparative(&self.lemma));
                self.forms.push(adjective_superlative(&self.lemma));
            }
            WordClass::Noun if self.has_plural() => {
                self.forms.push(noun_plural(&self.lemma));
            }
            WordClass::Verb => {
                self.forms.push(verb_present(&self.lemma));
                self.forms.push(verb_present_participle(&self.lemma));
                self.forms.push(verb_past(&self.lemma));
            }
            _ => (),
        }
    }
}

/// Make a regular plural noun from the singular form
fn noun_plural(lemma: &str) -> String {
    if ends_in_y(lemma) {
        let root = lemma.trim_end_matches('y');
        format!("{root}ies")
    } else if lemma.ends_with("s")
        || lemma.ends_with("sh")
        || lemma.ends_with("ch")
        || lemma.ends_with("x")
        || lemma.ends_with("z")
    {
        format!("{lemma}es")
    } else {
        format!("{lemma}s")
    }
}

/// Check if a character is a vowel
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Check if a word ends with a consonant which should repeat
fn consonant_end_repeat(s: &str) -> Option<char> {
    // consonant doubling rules (as far as I can tell):
    // 1. stress on final syllable
    // 2. always double an "l" final consonant (not "refuel", "parallel")
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

/// Make a regular present verb from the lemma form
fn verb_present(lemma: &str) -> String {
    if ends_in_y(lemma) {
        let root = lemma.trim_end_matches('y');
        format!("{root}ies")
    } else if lemma.ends_with("s") || lemma.ends_with("z") {
        match consonant_end_repeat(lemma) {
            Some(end) => format!("{lemma}{end}es"),
            None => format!("{lemma}es"),
        }
    } else if lemma.ends_with("sh")
        || lemma.ends_with("ch")
        || lemma.ends_with("x")
    {
        format!("{lemma}es")
    } else {
        format!("{lemma}s")
    }
}

/// Check if a lemma word ends in `y` (with no other vowel)
fn ends_in_y(lemma: &str) -> bool {
    lemma.ends_with("y")
        && !(lemma.ends_with("ay")
            || lemma.ends_with("ey")
            || lemma.ends_with("iy")
            || lemma.ends_with("oy")
            || lemma.ends_with("uy")
            || lemma.ends_with("yy"))
}

/// Check if a lemma word ends in `e`
fn ends_in_e(lemma: &str) -> bool {
    lemma.ends_with("e")
        && !(lemma.ends_with("ae")
            || lemma.ends_with("ee")
            || lemma.ends_with("ie")
            || lemma.ends_with("oe")
            || lemma.ends_with("ye"))
}

/// Make a regular present participle verb from the lemma form
fn verb_present_participle(lemma: &str) -> String {
    if let Some(end) = consonant_end_repeat(lemma) {
        return format!("{lemma}{end}ing");
    }
    if ends_in_e(lemma) {
        let root = lemma.trim_end_matches('e');
        format!("{root}ing")
    } else {
        format!("{lemma}ing")
    }
}

/// Make a regular past verb from the lemma form
fn verb_past(lemma: &str) -> String {
    if let Some(end) = consonant_end_repeat(lemma) {
        return format!("{lemma}{end}ed");
    }
    if lemma.ends_with("e") {
        format!("{lemma}d")
    } else if ends_in_y(lemma) {
        let root = lemma.trim_end_matches('y');
        format!("{root}ied")
    } else {
        format!("{lemma}ed")
    }
}

/// Make a regular comparative adjective from the lemma form
fn adjective_comparative(lemma: &str) -> String {
    if lemma.ends_with("e") {
        return format!("{lemma}r");
    } else if ends_in_y(lemma) {
        let root = lemma.trim_end_matches('y');
        return format!("{root}ier");
    }
    match consonant_end_repeat(lemma) {
        Some(end) => format!("{lemma}{end}er"),
        None => format!("{lemma}er"),
    }
}

/// Make a regular superlative adjective from the lemma form
fn adjective_superlative(lemma: &str) -> String {
    if lemma.ends_with("e") {
        return format!("{lemma}st");
    } else if ends_in_y(lemma) {
        let root = lemma.trim_end_matches('y');
        return format!("{root}iest");
    }
    match consonant_end_repeat(lemma) {
        Some(end) => format!("{lemma}{end}est"),
        None => format!("{lemma}est"),
    }
}
