use anyhow::Result;
use argh::FromArgs;
use booky::hilite;
use booky::kind::Kind;
use booky::lex;
use booky::tally::WordTally;
use booky::word::{Lexeme, WordClass};
use std::io::{IsTerminal, stdin};
use yansi::{Paint, Style};

/// Command-line arguments
#[derive(FromArgs, Debug, PartialEq)]
struct Args {
    #[argh(subcommand)]
    cmd: Option<SubCommand>,
}

/// Sub-command enum
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum SubCommand {
    Hilite(HiliteCmd),
    Kind(KindCmd),
    Lex(LexCmd),
    Nonsense(Nonsense),
}

/// Hilight text from stdin
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "hl")]
struct HiliteCmd {}

/// Group words by kind from stdin
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "kind")]
struct KindCmd {
    /// list words of all kinds
    #[argh(switch, short = 'A')]
    all: bool,
    /// list lexicon words
    #[argh(switch, short = 'l')]
    lexicon: bool,
    /// list foreign words (non-English)
    #[argh(switch, short = 'f')]
    foreign: bool,
    /// list ordinal numbers
    #[argh(switch, short = 'o')]
    ordinal: bool,
    /// list roman numerals
    #[argh(switch, short = 'r')]
    roman: bool,
    /// list numbers
    #[argh(switch, short = 'n')]
    number: bool,
    /// list acronyms / initialisms
    #[argh(switch, short = 'a')]
    acronym: bool,
    /// list proper names
    #[argh(switch, short = 'p')]
    proper: bool,
    /// list symbols / letters
    #[argh(switch, short = 's')]
    symbol: bool,
    /// list unknown words
    #[argh(switch, short = 'u')]
    unknown: bool,
}

/// List words from lexicon
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "lex")]
struct LexCmd {
    /// list all word forms
    #[argh(switch, short = 'f')]
    forms: bool,
    /// lookup a word
    #[argh(positional)]
    word: Option<String>,
}

/// Generate nonsense text
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "nonsense")]
struct Nonsense {}

impl HiliteCmd {
    /// Run command
    fn run(self) -> Result<()> {
        let stdin = stdin();
        if stdin.is_terminal() {
            eprintln!(
                "{0} stdin must be redirected {0}",
                "!!!".bright_yellow()
            );
            return Ok(());
        }
        hilite::hilite_text(stdin.lock())?;
        Ok(())
    }
}

impl KindCmd {
    /// Run command
    fn run(self) -> Result<()> {
        let stdin = stdin();
        if stdin.is_terminal() {
            eprintln!(
                "{0} stdin must be redirected {0}",
                "!!!".bright_yellow()
            );
            return Ok(());
        }
        let mut tally = WordTally::new();
        tally.parse_text(stdin.lock())?;
        if Kind::all().iter().any(|k| self.show_kind(*k)) {
            self.write_entries(tally)
        } else {
            self.write_summary(tally)
        }
    }

    /// Check if a word kind should be shown
    fn show_kind(&self, kind: Kind) -> bool {
        if self.all {
            return true;
        }
        match kind {
            Kind::Lexicon => self.lexicon,
            Kind::Foreign => self.foreign,
            Kind::Ordinal => self.ordinal,
            Kind::Roman => self.roman,
            Kind::Number => self.number,
            Kind::Acronym => self.acronym,
            Kind::Proper => self.proper,
            Kind::Symbol => self.symbol,
            Kind::Unknown => self.unknown,
        }
    }

    /// Write entries of selected kinds
    fn write_entries(self, tally: WordTally) -> Result<()> {
        let mut count = 0;
        for entry in tally.into_entries() {
            if self.show_kind(entry.kind()) {
                println!("{entry}");
                count += 1;
            }
        }
        println!("\ncount: {}", count.bright_yellow());
        Ok(())
    }

    /// Write summary of kinds
    fn write_summary(self, tally: WordTally) -> Result<()> {
        for kind in Kind::all() {
            let count = tally.count_kind(*kind);
            println!(
                "{:5} {} {kind:?}",
                count.bright_yellow(),
                kind.code().yellow()
            );
        }
        Ok(())
    }
}

impl LexCmd {
    /// Run command
    fn run(self) -> Result<()> {
        if self.forms {
            let mut forms: Vec<_> = lex::builtin().forms().collect();
            forms.sort();
            for form in forms {
                println!("{form}");
            }
        } else if let Some(word) = &self.word {
            self.lookup(word)?;
        } else {
            // into_iter() sorts the entries
            for word in lex::builtin().clone().into_iter() {
                println!("{word:?}");
            }
        }
        Ok(())
    }

    /// Lookup a word form
    fn lookup(&self, word: &str) -> Result<()> {
        let lex = lex::builtin();
        if lex.contains(word) {
            for w in lex.word_entries(word) {
                for f in w.forms() {
                    let mut style = if f == word {
                        Style::new().bright_yellow().italic()
                    } else {
                        Style::new()
                    };
                    if f == w.lemma() {
                        style = style.bold();
                        print!("{}:{} ", f.paint(style), w.word_class().bold());
                    } else {
                        print!("{} ", f.paint(style));
                    }
                }
                println!();
            }
        } else {
            println!("`{word}` not found");
        }
        Ok(())
    }
}

/// Choose a word from a slice
fn choose_word<'a>(words: &[&'a Lexeme]) -> &'a Lexeme {
    let mut n = words.len();
    n = fastrand::usize(1..=n);
    n = fastrand::usize(..n);
    words.get(n).unwrap()
}

/// Print nonsense
fn nonsense() {
    let nouns: Vec<_> = lex::builtin()
        .iter()
        .filter(|w| w.word_class() == WordClass::Noun)
        .collect();
    let verbs: Vec<_> = lex::builtin()
        .iter()
        .filter(|w| w.word_class() == WordClass::Verb)
        .collect();
    let subject = choose_word(&nouns[..]).lemma();
    let verb = choose_word(&verbs[..]).lemma();
    println!("{subject} {verb}")
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    match args.cmd {
        Some(SubCommand::Hilite(cmd)) => cmd.run()?,
        Some(SubCommand::Kind(cmd)) => cmd.run()?,
        Some(SubCommand::Lex(cmd)) => cmd.run()?,
        Some(SubCommand::Nonsense(_)) => nonsense(),
        None => {
            if let Err(e) = Args::from_args(&["booky"], &["--help"]) {
                eprintln!("{}", e.output);
            }
        }
    }
    Ok(())
}
