use crate::utils::{self, BaseOptions};
use clap::ArgMatches;
use globset::GlobSet;
use std::error::Error;
use std::path::PathBuf;

pub struct ExtractArchive {
    base: BaseOptions,
    patterns: GlobSet,
    output: PathBuf,
}

impl ExtractArchive {
    pub fn read(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let output = match matches.value_of_os("OUTPUT") {
            Some(x) => x.into(),
            None => std::env::current_dir()?,
        };

        Ok(ExtractArchive {
            base: BaseOptions::read(matches),
            patterns: utils::create_globset(matches, "patterns")?,
            output,
        })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        self.base
            .vprintln(format_args!("Reading archive from: {}", self.base.file));
        let archive = self.base.file.read_archive()?;

        self.base
            .vprintln(format_args!("Writing to: {}", self.base.file));
        std::fs::create_dir_all(&self.output)?;

        for (path, data) in archive.iter() {
            if !self.patterns.is_empty() && !self.patterns.is_match(path) {
                continue;
            }

            let target = self.output.join(path);
            match data {
                Some(data) => {
                    self.base
                        .vprintln(format_args!("Extracting file: {}", target.display()));
                    std::fs::write(target, data)?;
                }
                None => {
                    self.base
                        .vprintln(format_args!("Creating directory: {}", target.display()));
                    std::fs::create_dir(target)?;
                }
            }
        }

        Ok(())
    }
}
