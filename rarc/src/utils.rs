use clap::ArgMatches;
use globset::{Glob, GlobSet, GlobSetBuilder};
use jsystem::rarc::RarcArchive;
use std::error::Error;
use std::fmt::{self, Display, Write};
use std::io::{Cursor, Read};
use std::path::PathBuf;

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(PartialEq, Eq)]
pub enum FileInput {
    Stdin,
    Path(PathBuf),
}

impl FileInput {
    pub fn read(matches: &ArgMatches, name: &str) -> Self {
        match matches.value_of_os(name) {
            Some(x) => {
                if x == "-" {
                    FileInput::Stdin
                } else {
                    FileInput::Path(x.into())
                }
            }
            None => FileInput::Stdin,
        }
    }

    pub fn read_archive(&self) -> BoxResult<RarcArchive> {
        match self {
            FileInput::Path(path) => {
                let file = std::fs::File::open(path)?;
                Ok(RarcArchive::read(file)?)
            }
            FileInput::Stdin => {
                let mut buffer = Vec::new();
                let mut stdin = std::io::stdin();
                stdin.read_to_end(&mut buffer)?;
                Ok(RarcArchive::read(Cursor::new(buffer))?)
            }
        }
    }
}

impl Display for FileInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FileInput::*;
        match self {
            Stdin => f.write_char('-'),
            Path(path) => path.display().fmt(f),
        }
    }
}

pub struct BaseOptions {
    pub file: FileInput,
    pub verbose: bool,
}

impl BaseOptions {
    pub fn read(matches: &ArgMatches) -> Self {
        BaseOptions {
            file: FileInput::read(matches, "FILE"),
            verbose: matches.is_present("verbose"),
        }
    }

    #[inline]
    pub fn vprintln(&self, args: fmt::Arguments) {
        if self.verbose {
            if self.file == FileInput::Stdin {
                eprintln!("{}", args);
            } else {
                println!("{}", args);
            }
        }
    }
}

pub fn create_globset(matches: &ArgMatches, name: &str) -> BoxResult<GlobSet> {
    Ok(match matches.values_of(name) {
        Some(values) => {
            let mut patterns = GlobSetBuilder::new();
            for value in values {
                patterns.add(Glob::new(value)?);
            }
            patterns.build()?
        }
        None => GlobSet::empty(),
    })
}
