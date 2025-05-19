/// Word contractions
enum Contraction {
    Full(&'static str, &'static str, &'static str),
    Prefix(&'static str, &'static str),
    Suffix(&'static str, &'static str),
}

/// Some contractions
const CONTRACTIONS: &[Contraction] = &[
    Contraction::Full("ain’t", "am", "not"),
    Contraction::Full("can’t", "can", "not"),
    Contraction::Full("shan’t", "shall", "not"),
    Contraction::Full("won’t", "will", "not"),
    Contraction::Suffix("n’t", "not"),
    Contraction::Suffix("’ve", "have"),
    Contraction::Suffix("’ll", "will"),
    Contraction::Full("I’m", "I", "am"),
    Contraction::Suffix("’re", "are"),
    Contraction::Full("he’s", "he", "is"),
    Contraction::Full("it’s", "it", "is"),
    Contraction::Full("she’s", "she", "is"),
    Contraction::Full("that’s", "that", "is"),
    Contraction::Full("there’s", "there", "is"),
    Contraction::Full("what’s", "what", "is"),
    Contraction::Full("who’s", "who", "is"),
    Contraction::Full("’tis", "it", "is"),
    Contraction::Full("’twas", "it", "was"),
    Contraction::Full("’twill", "it", "will"),
    Contraction::Full("m’dear", "my", "dear"),
    Contraction::Full("m’lady", "my", "lady"),
    Contraction::Full("m’lord", "my", "lord"),
    Contraction::Suffix("’d", "would"),
    Contraction::Suffix("’s", ""), // possessive
    Contraction::Suffix("’", ""),  // possessive
    Contraction::Prefix("’", "’"), // nested quote
];

impl Contraction {
    /// Try to expand the contraction
    fn try_expand<'a>(&self, word: &'a str) -> Option<(&'a str, &'a str)> {
        match self {
            Contraction::Full(c, a, b) => {
                if equals_contraction(c, word) {
                    return Some((a, b));
                }
            }
            Contraction::Prefix(p, ex) => {
                let len = p.chars().count();
                if let Some((i, _c)) = word.char_indices().nth(len) {
                    if let Some((a, b)) = word.split_at_checked(i) {
                        if equals_contraction(p, a) {
                            return Some((b, ex));
                        }
                    }
                }
            }
            Contraction::Suffix(s, ex) => {
                let len = s.chars().count() - 1;
                if let Some((i, _c)) = word.char_indices().rev().nth(len) {
                    if let Some((a, b)) = word.split_at_checked(i) {
                        if equals_contraction(s, b) {
                            return Some((ex, a));
                        }
                    }
                }
            }
        }
        None
    }
}

/// Check if a contraction part equals a string
fn equals_contraction(part: &str, word: &str) -> bool {
    if part.chars().count() != word.chars().count() {
        return false;
    }
    for (a, b) in part.chars().zip(word.chars()) {
        let a = a.to_ascii_lowercase();
        let b = b.to_ascii_lowercase();
        if a != b && !(is_apostrophe(a) && is_apostrophe(b)) {
            return false;
        }
    }
    true
}

/// Check if a character is an apostrophe
///
/// Unicode has several different apostrophes:
///  - ' `U+0027` (ASCII APOSTROPHE)
///  - ʼ `U+02BC` (MODIFIER LETTER APOSTROPHE) -- glottal stop
///  - ’ `U+2019` (RIGHT SINGLE QUOTATION MARK) -- recommended by Unicode!
///  - ＇ `U+FF07` (FULLWIDTH APOSTROPHE)
fn is_apostrophe(c: char) -> bool {
    c == '\u{0027}' || c == '\u{02BC}' || c == '\u{2019}' || c == '\u{FF07}'
}

/// Split contractions
pub fn split(word: &str) -> Vec<&str> {
    let mut words = vec![word];
    let mut ex = Vec::with_capacity(2);
    while let Some(word) = words.pop() {
        if let Some(word) = split_contraction(&mut words, word) {
            ex.push(word);
        }
    }
    ex
}

/// Split one contraction
fn split_contraction<'a>(
    words: &mut Vec<&'a str>,
    word: &'a str,
) -> Option<&'a str> {
    for con in CONTRACTIONS {
        if let Some(ex) = con.try_expand(word) {
            words.push(ex.0);
            words.push(ex.1);
            return None;
        }
    }
    Some(word)
}
