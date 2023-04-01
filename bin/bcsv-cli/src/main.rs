mod decoder;
mod encoder;

use bcsv::byteorder::{BigEndian, LittleEndian};
use bcsv::Table;
use clap::Parser;
use color_eyre::Result;
use csv::{Reader, StringRecord};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Cursor, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "BCSV Tool")]
#[command(about = "Convert between Nintendo BCSV and normal CSV files", long_about = None)]
#[command(author, version)]
pub struct AppSettings {
    /// Input file
    input: Option<PathBuf>,

    /// Output file
    output: Option<PathBuf>,

    /// Tires to crack field name hashes based on the given file
    #[arg(long, short)]
    crack: Option<PathBuf>,

    /// Turns the tool into decode mode
    #[arg(long, short)]
    decode: bool,

    /// Will use little endain encoding instead of big endian
    #[arg(long, short)]
    little_endian: bool,
}

impl AppSettings {
    pub fn read_input_bcsv(&self) -> Result<Table> {
        match &self.input {
            Some(x) => Ok(if self.little_endian {
                Table::open::<LittleEndian>(x)?
            } else {
                Table::open::<BigEndian>(x)?
            }),
            None => {
                let mut stdin = io::stdin();
                let mut buffer = Vec::new();
                stdin.read_to_end(&mut buffer)?;
                Ok(if self.little_endian {
                    Table::read::<LittleEndian>(Cursor::new(buffer))?
                } else {
                    Table::read::<BigEndian>(Cursor::new(buffer))?
                })
            }
        }
    }

    pub fn read_input_csv(&self) -> Result<(StringRecord, Vec<StringRecord>)> {
        match &self.input {
            Some(x) => {
                let mut reader = Reader::from_path(x)?;
                let header = reader.headers()?.clone();
                let mut body = Vec::new();
                for row in reader.into_records() {
                    body.push(row?);
                }
                Ok((header, body))
            }
            None => {
                let mut reader = Reader::from_reader(std::io::stdin());
                let header = reader.headers()?.clone();
                let mut body = Vec::new();
                for row in reader.into_records() {
                    body.push(row?);
                }
                Ok((header, body))
            }
        }
    }

    pub fn create_crack_map(&self) -> Result<HashMap<u32, String>> {
        let path = match &self.crack {
            Some(path) => path,
            None => return Ok(HashMap::new()),
        };

        let file = BufReader::new(fs::File::open(path)?);
        let mut result = HashMap::new();
        for line in file.lines() {
            let line = line?;

            if line.starts_with('#') {
                continue;
            }

            result.insert(bcsv::jgadget_hash(line.as_bytes()), line);
        }

        Ok(result)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let options = AppSettings::parse();

    if options.decode {
        decoder::decode(options)?;
    } else {
        encoder::encode(options)?;
    };

    Ok(())
}
