use crate::utils::{self, BaseOptions};
use clap::ArgMatches;
use globset::GlobSet;
use humansize::{file_size_opts, FileSize};
use jsystem::rarc::RarcArchive;
use std::error::Error;

const FILE_SIZE_OPTIONS: file_size_opts::FileSizeOpts = file_size_opts::FileSizeOpts {
    space: false,
    ..file_size_opts::CONVENTIONAL
};

pub struct ListArchive {
    base: BaseOptions,
    patterns: GlobSet,
    humansize: bool,
}

impl ListArchive {
    pub fn read(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        Ok(ListArchive {
            base: BaseOptions::read(matches),
            patterns: utils::create_globset(matches, "patterns")?,
            humansize: matches.is_present("humansize"),
        })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let archive = self.base.file.read_archive()?;
        if self.base.verbose {
            self.print_verbose(archive);
        } else {
            self.print_simple(archive);
        }

        Ok(())
    }

    fn print_simple(self, archive: RarcArchive) {
        for (path, _) in archive.iter() {
            if !self.patterns.is_empty() && !self.patterns.is_match(path) {
                continue;
            }

            println!("{}", path);
        }
    }

    fn print_verbose(self, archive: RarcArchive) {
        let mut collected = Vec::new();
        for (path, data) in archive.iter() {
            if !self.patterns.is_empty() && !self.patterns.is_match(path) {
                continue;
            }

            let ty = if data.is_some() { '-' } else { 'd' };
            let len = data.map(|x| x.len()).unwrap_or_default();
            let size = if self.humansize {
                let mut size = match len.file_size(FILE_SIZE_OPTIONS) {
                    Ok(x) => x,
                    Err(x) => x,
                };

                let mut i = size.chars();
                i.next_back();
                if let Some(x) = i.next_back() {
                    if x.is_numeric() {
                        size.push(' ');
                    }
                }

                size
            } else {
                format!("{}", len)
            };

            collected.push((ty, size, path));
        }

        if collected.is_empty() {
            return;
        }

        let max_len = collected
            .iter()
            .map(|(_, size, _)| size.len())
            .max()
            .unwrap();
        for (ty, size, path) in collected.iter() {
            println!("{} {:>3$} {}", ty, size, path, max_len);
        }
    }
}
