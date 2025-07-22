use deunicode::deunicode_char;
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
    /// `z` Alternate `z => s` spelling
    AlternateZ,
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
    /// Irregular forms (encoded)
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

impl WordClass {
    /// Build regular inflected forms
    fn build_regular_forms(self, lex: &Lexeme, lemma: &str) -> Vec<String> {
        let mut forms = Vec::new();
        match self {
            WordClass::Adjective if lex.count_syllables() < 4 => {
                forms.push(adjective_comparative(lemma));
                forms.push(adjective_superlative(lemma));
            }
            WordClass::Noun if lex.has_plural() => {
                forms.push(noun_plural(lemma));
            }
            WordClass::Verb => {
                forms.push(verb_present(lemma));
                forms.push(verb_present_participle(lemma));
                forms.push(verb_past(lemma));
            }
            _ => (),
        }
        forms
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
            'z' => Ok(Self::AlternateZ),
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
            let form = decode_irregular(&lemma, form)?;
            let form = encode_irregular(&lemma, &form);
            irregular_forms.push(form);
        }
        let forms = Vec::new();
        let mut word = Lexeme {
            lemma,
            word_class,
            attr,
            irregular_forms,
            forms,
        };
        word.build_inflected_forms()?;
        Ok(word)
    }
}

/// Decode an irregular word form
fn decode_irregular(lemma: &str, form: &str) -> Result<String, ()> {
    if let Some(suffix) = form.strip_prefix('-') {
        if let Some(ch) = suffix.chars().next() {
            if let Some((base, _ending)) = lemma.rsplit_once(ch) {
                let mut f = String::with_capacity(base.len() + suffix.len());
                f.push_str(base);
                f.push_str(suffix);
                return Ok(f);
            }
            // check for variant spelling of suffix joiner
            if let Some(alt) = deunicode_char(ch) {
                if alt.chars().nth(0) != Some(ch) {
                    if let Some((base, _ending)) = lemma.rsplit_once(alt) {
                        let mut f =
                            String::with_capacity(base.len() + suffix.len());
                        let mut suffix = suffix.chars();
                        suffix.next(); // skip joiner character
                        f.push_str(base);
                        f.push_str(alt);
                        f.push_str(suffix.as_str());
                        return Ok(f);
                    }
                }
            }
            return Err(());
        }
    }
    Ok(form.into())
}

/// Encode an irregular word form
fn encode_irregular(lemma: &str, form: &str) -> String {
    let mut pos = None;
    for i in 3..lemma.len() {
        if let (Some((a0, a1)), Some((b0, b1))) =
            (lemma.split_at_checked(i), form.split_at_checked(i))
        {
            if a0 == b0 {
                let mut ch = a1.chars();
                if let Some(c) = ch.next() {
                    if b1.starts_with(c) && !ch.any(|x| x == c) {
                        pos = Some(i);
                    }
                }
            }
        }
    }
    if let Some(i) = pos {
        let suffix = &form[i..];
        let mut s = String::from('-');
        s.push_str(suffix);
        return s;
    }
    form.into()
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.lemma, self.word_class)?;
        if !self.attr.is_empty() {
            write!(fmt, ".{}", self.attr)?;
        }
        for form in &self.irregular_forms {
            write!(fmt, ",{form}")?;
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

    /// Check if a word has alternate spelling form (`z => s`)
    fn has_alternate_z(&self) -> bool {
        for a in self.attr.chars() {
            if let Ok(WordAttr::AlternateZ) = WordAttr::try_from(a) {
                return true;
            }
        }
        false
    }

    /// Build inflected word forms
    fn build_inflected_forms(&mut self) -> Result<(), ()> {
        for variant in self.variant_spellings() {
            self.build_inflected(&variant)?;
        }
        Ok(())
    }

    /// Get all variant spellings of the lemma
    fn variant_spellings(&self) -> Vec<String> {
        let mut variants = Vec::new();
        variants.push(String::new());
        for ch in self.lemma.chars() {
            if let Some(alt) = deunicode_char(ch) {
                let mut more = Vec::new();
                if alt.chars().nth(0) != Some(ch) {
                    for variant in &variants {
                        let mut v = variant.to_string();
                        v.push_str(alt);
                        more.push(v);
                    }
                }
                if ch == 'æ' || ch == 'œ' {
                    for variant in &variants {
                        let mut v = variant.to_string();
                        v.push('e');
                        more.push(v);
                    }
                }
                for variant in variants.iter_mut() {
                    variant.push(ch);
                }
                variants.extend(more);
            }
        }
        if self.has_alternate_z() {
            let mut more = Vec::new();
            for variant in &variants {
                more.push(variant.replace('z', "s"));
            }
            variants.extend(more);
        }
        variants
    }

    /// Build inflected word forms
    fn build_inflected(&mut self, lemma: &str) -> Result<(), ()> {
        self.forms.push(lemma.to_string());
        if self.irregular_forms.is_empty() {
            self.forms
                .extend(self.word_class.build_regular_forms(self, lemma));
        } else {
            for form in &self.irregular_forms {
                let form = decode_irregular(lemma, form)?;
                if form != lemma {
                    self.forms.push(form);
                }
            }
        }
        Ok(())
    }
}

/// Make a regular plural noun from the singular form
fn noun_plural(lemma: &str) -> String {
    if let Some(root) = lemma.strip_suffix("sis") {
        if !root.is_empty() {
            return format!("{root}ses");
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn variants() {
        let lex = Lexeme::try_from("café:N").unwrap();
        assert_eq!(lex.variant_spellings(), vec!["café", "cafe",]);
        let lex = Lexeme::try_from("façade:N").unwrap();
        assert_eq!(lex.variant_spellings(), vec!["façade", "facade",]);
        let lex = Lexeme::try_from("appliqué:V,-és,-éing,-éd").unwrap();
        assert_eq!(lex.variant_spellings(), vec!["appliqué", "applique"]);
        let lex = Lexeme::try_from("anæsthetize:V.z").unwrap();
        assert_eq!(
            lex.variant_spellings(),
            vec![
                "anæsthetize",
                "anaesthetize",
                "anesthetize",
                "anæsthetise",
                "anaesthetise",
                "anesthetise",
            ]
        );
    }

    #[test]
    fn irregular() {
        let a = decode_irregular("addendum", "-da").unwrap();
        let form = encode_irregular("addendum", &a);
        assert_eq!(form, "-da");
    }
}
