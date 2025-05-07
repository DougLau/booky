use anyhow::Result;
use argh::FromArgs;
use booky::tally::{Category, WordTally};
use booky::word::{Dict, Word, WordClass};
use std::io::{BufWriter, Write, stdin, stdout};
use yansi::Paint;

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
    Freq(Freq),
    Nonsense(Nonsense),
}

/// Categorize words
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "cat")]
struct CatCmd {
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

/// Print dictionary words
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

/// Count word frequencies
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "freq")]
struct Freq {}

/// Generate nonsense text
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "nonsense")]
struct Nonsense {}

impl CatCmd {
    /// Run command
    fn run(self) -> Result<()> {
        if Category::all().iter().any(|c| self.show_category(*c)) {
            self.list_category()?;
        } else {
            self.counts()?;
        }
        Ok(())
    }

    /// Count words of categories
    fn counts(self) -> Result<()> {
        let builtin = Dict::builtin();
        let mut tally = WordTally::new();
        tally.parse_text(stdin().lock())?;
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

    /// List words of selected categories
    fn list_category(self) -> Result<()> {
        let builtin = Dict::builtin();
        let mut tally = WordTally::new();
        tally.parse_text(stdin().lock())?;
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

    /// Check if a word category should be shown
    fn show_category(&self, cat: Category) -> bool {
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
                        write!(writer, "{} ", form.italic())?;
                        for f in w.forms() {
                            if f == word {
                                write!(
                                    writer,
                                    "{} ",
                                    f.bright().green().underline()
                                )?;
                            } else {
                                write!(writer, "{f} ")?;
                            }
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

/// Count word frequency in text
fn freq() -> Result<()> {
    let builtin = Dict::builtin();
    let mut tally = WordTally::new();
    tally.parse_text(stdin().lock())?;
    tally.split_unknown_compounds(&builtin);
    tally.split_unknown_contractions(&builtin);
    tally.check_dict(&builtin);
    let mut writer = BufWriter::new(stdout());
    let mut count = 0;
    for entry in tally.into_entries() {
        writeln!(writer, "{entry}")?;
        count += 1;
    }
    writeln!(writer, "\ncount: {}", count.bright().yellow())?;
    Ok(())
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
        Some(SubCommand::Freq(_)) => freq()?,
        Some(SubCommand::Nonsense(_)) => nonsense(),
        None => {
            if let Err(e) = Args::from_args(&["booky"], &["--help"]) {
                eprintln!("{}", e.output);
            }
        }
    }
    Ok(())
}
