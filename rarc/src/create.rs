use crate::utils::{self, BaseOptions, FileInput};
use clap::ArgMatches;
use globset::GlobSet;
use jsystem::rarc::RarcArchive;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;

#[cfg(unix)]
fn archive_path(input: &OsStr) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let buffer = input.as_bytes();
    if buffer[0] == b'@' {
        Some(OsStr::from_bytes(&buffer[1..]).into())
    } else {
        None
    }
}

enum CreateFileInput {
    File(PathBuf),
    Archive(PathBuf),
}

impl CreateFileInput {
    fn read(matches: &ArgMatches, name: &str) -> Vec<Self> {
        match matches.values_of_os(name) {
            Some(values) => {
                let mut result = Vec::with_capacity(values.len());
                for value in values {
                    if let Some(archive) = archive_path(value) {
                        result.push(Self::Archive(archive));
                    } else {
                        result.push(Self::File(value.into()));
                    }
                }
                result
            }
            None => Vec::new(),
        }
    }
}

pub struct CreateArchive {
    base: BaseOptions,
    level: usize,
    exclude: GlobSet,
    input: Vec<CreateFileInput>,
}

fn clamp(value: usize, min: usize, max: usize) -> usize {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

impl CreateArchive {
    pub fn read(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let level = match matches.value_of("LEVEL") {
            Some(l) => l.parse::<usize>()?,
            None => 10,
        };
        Ok(CreateArchive {
            base: BaseOptions::read(matches),
            level: clamp(level, 1, 10),
            exclude: utils::create_globset(matches, "EXCLUDE")?,
            input: CreateFileInput::read(matches, "INPUT"),
        })
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let mut output = RarcArchive::default();

        for input in self.input.iter() {
            match input {
                CreateFileInput::File(path) => {
                    if self.exclude.is_match(path) {
                        continue;
                    }

                    let buffer = std::fs::read(path)?;

                    let path_str = match path.to_str() {
                        Some(s) => s,
                        None => {
                            eprintln!("Cannot insert file with borken path: {}", path.display());
                            continue;
                        }
                    };

                    self.base
                        .vprintln(format_args!("Adding file: {}", path_str));
                    output.create_file(path_str, buffer.into());
                }
                CreateFileInput::Archive(path) => {
                    let file = std::fs::File::open(path)?;
                    let archive = RarcArchive::read(file)?;

                    self.base
                        .vprintln(format_args!("Merging archive: {}", path.display()));
                    for (path, data) in archive.iter() {
                        if self.exclude.is_match(path) {
                            continue;
                        }

                        match data {
                            Some(data) => {
                                self.base.vprintln(format_args!("Adding file: {}", path));
                                output.create_file(path, data.clone());
                            }
                            None => {
                                self.base
                                    .vprintln(format_args!("Creating directory: {}", path));
                                output.create_directory(path);
                            }
                        }
                    }
                }
            }
        }

        match self.base.file {
            FileInput::Path(path) => {
                let mut file = std::fs::File::create(path)?;
                output.save(&mut file, self.level)?;
            },
            FileInput::Stdin => {
                let mut stdout = std::io::stdout();
                output.save(&mut stdout, self.level)?;
            }
        }

        Ok(())
    }
}
