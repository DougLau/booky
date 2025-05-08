use anyhow::Result;
use argh::FromArgs;
use booky::tally::{Category, WordTally};
use booky::word::{Dict, Word, WordClass};
use std::io::{BufRead, BufWriter, IsTerminal, Write, stdin, stdout};
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
    Cat(CatCmd),
    Dict(DictCmd),
    Nonsense(Nonsense),
}

/// Categorize words from stdin
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "cat")]
struct CatCmd {
    /// list words in all categories
    #[argh(switch, short = 'A')]
    all: bool,
    /// list dictionary words
    #[argh(switch, short = 'd')]
    dictionary: bool,
    /// list acronyms / initialisms
    #[argh(switch, short = 'a')]
    acronym: bool,
    /// list foreign words (non-English)
    #[argh(switch, short = 'f')]
    foreign: bool,
    /// list numbers
    #[argh(switch, short = 'n')]
    number: bool,
    /// list ordinal numbers
    #[argh(switch, short = 'o')]
    ordinal: bool,
    /// list proper names
    #[argh(switch, short = 'p')]
    proper: bool,
    /// list roman numerals
    #[argh(switch, short = 'r')]
    roman: bool,
    /// list single letter words
    #[argh(switch, short = 'l')]
    letter: bool,
    /// list unknown words
    #[argh(switch, short = 'u')]
    unknown: bool,
}

/// List words from dictionary
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "dict")]
struct DictCmd {
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

impl CatCmd {
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
        if Category::all().iter().any(|c| self.show_category(*c)) {
            self.list_category(stdin.lock())?;
        } else {
            self.counts(stdin.lock())?;
        }
        Ok(())
    }

    /// Check if a word category should be shown
    fn show_category(&self, cat: Category) -> bool {
        if self.all {
            return true;
        }
        match cat {
            Category::Dictionary => self.dictionary,
            Category::Acronym => self.acronym,
            Category::Foreign => self.foreign,
            Category::Ordinal => self.ordinal,
            Category::Number => self.number,
            Category::Proper => self.proper,
            Category::Roman => self.roman,
            Category::Letter => self.letter,
            Category::Unknown => self.unknown,
        }
    }

    /// List words of selected categories
    fn list_category<R: BufRead>(self, read: R) -> Result<()> {
        let builtin = Dict::builtin();
        let mut tally = WordTally::new();
        tally.parse_text(read)?;
        tally.split_unknown_compounds(&builtin);
        tally.split_unknown_contractions(&builtin);
        tally.check_dict(&builtin);
        let mut writer = BufWriter::new(stdout());
        let mut count = 0;
        for entry in tally.into_entries() {
            if self.show_category(entry.category()) {
                writeln!(writer, "{entry}")?;
                count += 1;
            }
        }
        writeln!(writer, "\ncount: {}", count.bright().yellow())?;
        Ok(())
    }

    /// Count words of categories
    fn counts<R: BufRead>(self, read: R) -> Result<()> {
        let builtin = Dict::builtin();
        let mut tally = WordTally::new();
        tally.parse_text(read)?;
        tally.split_unknown_compounds(&builtin);
        tally.split_unknown_contractions(&builtin);
        tally.check_dict(&builtin);
        let mut writer = BufWriter::new(stdout());
        for cat in Category::all() {
            let count = tally.cat_count(*cat);
            writeln!(
                writer,
                "{:5} {} {cat:?}",
                count.bright().yellow(),
                cat.code().yellow()
            )?;
        }
        Ok(())
    }
}

impl DictCmd {
    /// Run command
    fn run(self) -> Result<()> {
        if self.forms {
            let dict = Dict::builtin();
            let mut forms: Vec<_> = dict.forms().collect();
            forms.sort();
            for form in forms {
                println!("{form}");
            }
        } else if let Some(word) = &self.word {
            self.lookup(word)?;
        } else {
            let mut dict = Dict::builtin();
            dict.sort();
            for word in dict.iter() {
                println!("{word:?}");
            }
        }
        Ok(())
    }

    /// Lookup a word form
    fn lookup(&self, word: &str) -> Result<()> {
        let mut writer = BufWriter::new(stdout());
        let dict = Dict::builtin();
        if dict.contains(word) {
            for w in dict.iter() {
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
    let dict = Dict::builtin();
    let nouns: Vec<_> = dict
        .iter()
        .filter(|w| w.word_class() == Some(WordClass::Noun))
        .collect();
    let verbs: Vec<_> = dict
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
        Some(SubCommand::Cat(cmd)) => cmd.run()?,
        Some(SubCommand::Dict(cmd)) => cmd.run()?,
        Some(SubCommand::Nonsense(_)) => nonsense(),
        None => {
            if let Err(e) = Args::from_args(&["booky"], &["--help"]) {
                eprintln!("{}", e.output);
            }
        }
    }
    Ok(())
}
