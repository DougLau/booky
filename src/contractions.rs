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
    Contraction::Prefix("’", ""),  // weird quote
];

impl Contraction {
    /// Check if a word uses the contraction
    fn check(&self, word: &str) -> bool {
        match self {
            Contraction::Full(c, _, _) => word.eq_ignore_ascii_case(c),
            Contraction::Prefix(p, _) => word.starts_with(p),
            Contraction::Suffix(s, _) => word.ends_with(s),
        }
    }

    /// Expand the contraction
    fn expand<'a>(&self, word: &'a str) -> Vec<&'a str> {
        match self {
            Contraction::Full(_, a, b) => vec![a, b],
            Contraction::Prefix(p, ex) => match word.strip_prefix(p) {
                Some(base) => vec![base, ex],
                None => vec![word],
            },
            Contraction::Suffix(s, ex) => match word.strip_suffix(s) {
                Some(base) => vec![base, ex],
                None => vec![word],
            },
        }
    }
}

/// Split contractions
pub fn split(word: &str) -> Vec<&str> {
    let mut words = vec![word];
    let mut ex = Vec::with_capacity(2);
    while let Some(word) = words.pop() {
        let mut expanded = split_contraction(word);
        if expanded.is_empty() {
            ex.push(word);
        } else {
            words.append(&mut expanded);
        }
    }
    ex
}

/// Split one contraction
fn split_contraction(word: &str) -> Vec<&str> {
    for con in CONTRACTIONS {
        if con.check(word) {
            return con.expand(word);
        }
    }
    vec![]
}
