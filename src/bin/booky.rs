use anyhow::Result;
use argh::FromArgs;
use booky::kind::Kind;
use booky::tally::WordTally;
use booky::word::{Lexicon, Word, WordClass};
use std::io::{BufWriter, IsTerminal, Write, stdin, stdout};
use yansi::{Color::Green, Color::White, Paint};

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
    Kind(KindCmd),
    Lex(LexCmd),
    Nonsense(Nonsense),
}

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

impl KindCmd {
    /// Run command
    fn run(self) -> Result<()> {
        let stdin = stdin();
        if stdin.is_terminal() {
            eprintln!(
                "{0} stdin must be redirected {0}",
                "!!!".yellow().bright()
            );
            return Ok(());
        }
        let mut tally = WordTally::new(Lexicon::builtin());
        tally.parse_text(stdin.lock())?;
        tally.split_unknown_contractions();
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
        let mut writer = BufWriter::new(stdout());
        let mut count = 0;
        for entry in tally.into_entries() {
            if self.show_kind(entry.kind()) {
                writeln!(writer, "{entry}")?;
                count += 1;
            }
        }
        writeln!(writer, "\ncount: {}", count.bright().yellow())?;
        Ok(())
    }

    /// Write summary of kinds
    fn write_summary(self, tally: WordTally) -> Result<()> {
        let mut writer = BufWriter::new(stdout());
        for kind in Kind::all() {
            let count = tally.count_kind(*kind);
            writeln!(
                writer,
                "{:5} {} {kind:?}",
                count.bright().yellow(),
                kind.code().yellow()
            )?;
        }
        Ok(())
    }
}

impl LexCmd {
    /// Run command
    fn run(self) -> Result<()> {
        if self.forms {
            let lex = Lexicon::builtin();
            let mut forms: Vec<_> = lex.forms().collect();
            forms.sort();
            for form in forms {
                println!("{form}");
            }
        } else if let Some(word) = &self.word {
            self.lookup(word)?;
        } else {
            let mut lex = Lexicon::builtin();
            lex.sort();
            for word in lex.iter() {
                println!("{word:?}");
            }
        }
        Ok(())
    }

    /// Lookup a word form
    fn lookup(&self, word: &str) -> Result<()> {
        let mut writer = BufWriter::new(stdout());
        let lex = Lexicon::builtin();
        if lex.contains(word) {
            for w in lex.iter() {
                for form in w.forms() {
                    if form == word {
                        for f in w.forms() {
                            let mut style = if f == word {
                                Green.bright().underline().on_primary()
                            } else {
                                White.on_primary()
                            };
                            if f == w.base() {
                                style = style.italic();
                            }
                            write!(writer, "{} ", f.paint(style))?;
                        }
                        writeln!(writer)?;
                        break;
                    }
                }
            }
        } else {
            writeln!(writer, "`{word}` not found")?;
        }
        Ok(())
    }
}

/// Choose a word from a slice
fn choose_word<'a>(words: &'a [&'a Word]) -> &'a Word {
    let mut n = words.len();
    n = fastrand::usize(1..=n);
    n = fastrand::usize(..n);
    words.get(n).unwrap()
}

/// Print nonsense
fn nonsense() {
    let lex = Lexicon::builtin();
    let nouns: Vec<_> = lex
        .iter()
        .filter(|w| w.word_class() == Some(WordClass::Noun))
        .collect();
    let verbs: Vec<_> = lex
        .iter()
        .filter(|w| w.word_class() == Some(WordClass::Verb))
        .collect();
    let subject = choose_word(&nouns[..]).base();
    let verb = choose_word(&verbs[..]).base();
    println!("{subject} {verb}")
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    match args.cmd {
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
