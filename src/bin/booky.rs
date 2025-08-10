use anyhow::{Result, bail};
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
    Read(ReadCmd),
    Word(WordCmd),
    Nonsense(Nonsense),
}

/// Hilight text from stdin
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "hl")]
struct HiliteCmd {}

/// Read text from stdin, grouping tokens by kind
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "read")]
struct ReadCmd {
    /// token kinds (l,f,o,r,n,a,p,s,u,A)
    #[argh(positional)]
    kinds: Option<String>,
    /// token output limit
    #[argh(option, short = 't', default = "u32::MAX")]
    tokens: u32,
    /// reverse sort
    #[argh(switch, short = 'v')]
    reverse: bool,
    /// output token words only
    #[argh(switch, short = 'w')]
    word: bool,
}

/// Lookup words from lexicon
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "word")]
struct WordCmd {
    /// word classes (A,Av,C,D,I,N,P,Pn,V)
    #[argh(option, short = 'c')]
    classes: Option<String>,
    /// list all word forms
    #[argh(switch, short = 'f')]
    forms: bool,
    /// word to lookup
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

impl ReadCmd {
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
        let kinds = self.parse_kinds()?;
        let mut tally = WordTally::new();
        tally.parse_text(stdin.lock())?;
        if kinds.is_empty() {
            self.write_summary(tally)
        } else {
            self.write_entries(tally, &kinds)
        }
    }

    /// Parse token kinds
    fn parse_kinds(&self) -> Result<Vec<Kind>> {
        let mut kinds = Vec::new();
        if let Some(knd) = &self.kinds {
            for kind in knd.split(',') {
                let kind = match kind.trim() {
                    "A" => return Ok(Kind::all().to_vec()),
                    "l" => Kind::Lexicon,
                    "f" => Kind::Foreign,
                    "o" => Kind::Ordinal,
                    "r" => Kind::Roman,
                    "n" => Kind::Number,
                    "a" => Kind::Acronym,
                    "p" => Kind::Proper,
                    "s" => Kind::Symbol,
                    "u" => Kind::Unknown,
                    k => bail!("Unknown kind: {k}"),
                };
                kinds.push(kind);
            }
        }
        Ok(kinds)
    }

    /// Write entries of selected kinds
    fn write_entries(self, tally: WordTally, kinds: &[Kind]) -> Result<()> {
        let mut count = 0;
        let entries = if self.reverse {
            tally.into_entries()
        } else {
            tally.into_entries().into_iter().rev().collect()
        };
        for entry in entries {
            if kinds.contains(&entry.kind()) {
                if self.word {
                    println!("{}", entry.word());
                } else {
                    println!("{entry}");
                }
                count += 1;
            }
            if count >= self.tokens {
                break;
            }
        }
        if !self.word {
            println!("\ncount: {}", count.bright_yellow());
        }
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

impl WordCmd {
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
                if self.show_class(word.word_class()) {
                    println!("{word:?}");
                }
            }
        }
        Ok(())
    }

    /// Check if a word class should be shown
    fn show_class(&self, wc: WordClass) -> bool {
        match &self.classes {
            Some(classes) => {
                for cl in classes.split(',') {
                    match WordClass::try_from(cl) {
                        Ok(cl) => {
                            if cl == wc {
                                return true;
                            }
                        }
                        Err(_) => return false,
                    }
                }
                false
            }
            None => true,
        }
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
        Some(SubCommand::Read(cmd)) => cmd.run()?,
        Some(SubCommand::Word(cmd)) => cmd.run()?,
        Some(SubCommand::Nonsense(_)) => nonsense(),
        None => {
            if let Err(e) = Args::from_args(&["booky"], &["--help"]) {
                eprintln!("{}", e.output);
            }
        }
    }
    Ok(())
}
