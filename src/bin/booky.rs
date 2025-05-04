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
    Cat(Cat),
    Dict(DictCmd),
    Acronym(Acronym),
    Foreign(Foreign),
    Freq(Freq),
    List(List),
    Nonsense(Nonsense),
    Ordinal(Ordinal),
    Num(Num),
    Proper(Proper),
    RomanNum(RomanNum),
    Unknown(Unknown),
    Word(WordCmd),
}

/// Categorize words
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "cat")]
struct Cat {}

/// Print dictionary
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "dict")]
struct DictCmd {}

/// List acronyms / initialisms
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "acronym")]
struct Acronym {}

/// List foreign words
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "foreign")]
struct Foreign {}

/// Count word frequencies
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "freq")]
struct Freq {}

/// List all word forms
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "list")]
struct List {}

/// Generate nonsense text
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "nonsense")]
struct Nonsense {}

/// List ordinal numbers
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "ordinal")]
struct Ordinal {}

/// List numbers
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "num")]
struct Num {}

/// List proper nouns
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "proper")]
struct Proper {}

/// List roman numerals
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "roman")]
struct RomanNum {}

/// List unknown words
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "unknown")]
struct Unknown {}

/// Lookup a word
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "word")]
struct WordCmd {
    #[argh(positional)]
    word: String,
}

/// Count words of category
fn cat_counts() -> Result<()> {
    let builtin = Dict::builtin();
    let mut tally = WordTally::new();
    tally.parse_text(stdin().lock())?;
    tally.split_unknown_compounds(&builtin);
    tally.split_unknown_contractions(&builtin);
    tally.remove_single(&builtin);
    tally.remove_dict(&builtin);
    let mut writer = BufWriter::new(stdout());
    for cat in Category::all() {
        writeln!(writer, "{cat:?}: {}", tally.cat_count(*cat))?;
    }
    Ok(())
}

/// Print dictionary
fn dict() -> Result<()> {
    let mut dict = Dict::builtin();
    dict.sort();
    for word in dict.iter() {
        println!("{word:?}");
    }
    Ok(())
}

/// List all word forms
fn list() -> Result<()> {
    let dict = Dict::builtin();
    for form in dict.forms() {
        println!("{form}");
    }
    Ok(())
}

impl WordCmd {
    /// Lookup the word
    fn lookup(self) -> Result<()> {
        let mut writer = BufWriter::new(stdout());
        let dict = Dict::builtin();
        if dict.contains(&self.word) {
            for word in dict.iter() {
                for form in word.forms() {
                    if form == &self.word {
                        write!(writer, "{} ", word.italic())?;
                        for f in word.forms() {
                            if f == &self.word {
                                write!(
                                    writer,
                                    "{} ",
                                    f.underline().green().bright()
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
            writeln!(writer, "`{}` not found", self.word)?;
        }
        Ok(())
    }
}

/// List words of a given category
fn list_category(cat: Category) -> Result<()> {
    let builtin = Dict::builtin();
    let mut tally = WordTally::new();
    tally.parse_text(stdin().lock())?;
    tally.split_unknown_compounds(&builtin);
    tally.split_unknown_contractions(&builtin);
    tally.remove_single(&builtin);
    tally.retain_category(cat);
    let mut writer = BufWriter::new(stdout());
    let mut words = 0;
    writeln!(writer)?;
    for entry in tally.into_entries() {
        if !builtin.contains(entry.word()) {
            writeln!(writer, "{entry}")?;
            words += 1;
        }
    }
    writeln!(writer, "\n{cat:?}: {}", words.yellow().bright())?;
    Ok(())
}

/// Count word frequency in text
fn freq() -> Result<()> {
    let mut tally = WordTally::new();
    tally.parse_text(stdin().lock())?;
    let mut writer = BufWriter::new(stdout());
    let mut count = 0;
    for entry in tally.into_entries() {
        writeln!(writer, "{entry}")?;
        count += 1;
    }
    writeln!(writer, "\ncount: {count}")?;
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
        Some(SubCommand::Cat(_)) => cat_counts()?,
        Some(SubCommand::Dict(_)) => dict()?,
        Some(SubCommand::Acronym(_)) => list_category(Category::Acronym)?,
        Some(SubCommand::Foreign(_)) => list_category(Category::Foreign)?,
        Some(SubCommand::Freq(_)) => freq()?,
        Some(SubCommand::List(_)) => list()?,
        Some(SubCommand::Nonsense(_)) => nonsense(),
        Some(SubCommand::Num(_)) => list_category(Category::Number)?,
        Some(SubCommand::Ordinal(_)) => list_category(Category::Ordinal)?,
        Some(SubCommand::Proper(_)) => list_category(Category::Proper)?,
        Some(SubCommand::RomanNum(_)) => list_category(Category::Roman)?,
        Some(SubCommand::Unknown(_)) => list_category(Category::Unknown)?,
        Some(SubCommand::Word(word)) => word.lookup()?,
        None => {
            if let Err(e) = Args::from_args(&["booky"], &["--help"]) {
                eprintln!("{}", e.output);
            }
        }
    }
    Ok(())
}
