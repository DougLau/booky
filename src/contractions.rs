use crate::lex::is_apostrophe;

/// Word contractions
enum Contraction {
    Full(&'static str, &'static str, &'static str),
    Prefix(&'static str, &'static str),
    Suffix(&'static str, &'static str),
    SuffixReplacement(&'static str, &'static str),
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
    Contraction::SuffixReplacement("n’", "ng"),
    Contraction::Suffix("’", ""),  // possessive
    Contraction::Prefix("’", "’"), // nested quote
];

impl Contraction {
    /// Try to expand the contraction
    fn try_expand(&self, words: &mut Vec<String>, word: &str) -> bool {
        match self {
            Contraction::Full(c, a, b) => {
                if equals_contraction(c, word) {
                    words.push(a.to_string());
                    words.push(b.to_string());
                    return true;
                }
            }
            Contraction::Prefix(p, ex) => {
                let len = p.chars().count();
                if let Some((i, _c)) = word.char_indices().nth(len)
                    && let Some((a, b)) = word.split_at_checked(i)
                    && equals_contraction(p, a)
                {
                    words.push(b.to_string());
                    words.push(ex.to_string());
                    return true;
                }
            }
            Contraction::Suffix(s, ex) => {
                let len = s.chars().count() - 1;
                if let Some((i, _c)) = word.char_indices().rev().nth(len)
                    && let Some((a, b)) = word.split_at_checked(i)
                    && equals_contraction(s, b)
                {
                    words.push(ex.to_string());
                    words.push(a.to_string());
                    return true;
                }
            }
            Contraction::SuffixReplacement(s, ex) => {
                let len = s.chars().count() - 1;
                if let Some((i, _c)) = word.char_indices().rev().nth(len)
                    && let Some((a, b)) = word.split_at_checked(i)
                    && equals_contraction(s, b)
                {
                    let mut a = a.to_string();
                    a.push_str(ex);
                    words.push(a.to_string());
                    return true;
                }
            }
        }
        false
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

/// Split contractions
pub fn split(word: &str) -> Vec<String> {
    let mut words = vec![word.to_string()];
    let mut ex = Vec::with_capacity(2);
    while let Some(word) = words.pop() {
        if !split_contraction(&mut words, &word) {
            ex.push(word);
        }
    }
    ex
}

/// Split one contraction
fn split_contraction(words: &mut Vec<String>, word: &str) -> bool {
    for con in CONTRACTIONS {
        if con.try_expand(words, word) {
            return true;
        }
    }
    false
}
