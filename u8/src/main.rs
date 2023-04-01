mod extract;
mod list;
mod utils;

use clap::{App, AppSettings, Arg, ArgMatches};
use std::error::Error;

fn main() {
    let matches = App::new("U8 Tool")
        .author(clap::crate_authors!("\n"))
        .about("Manipulate Nintendo U8 archives")
        .version_short("V")
        .version(clap::crate_version!())
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            App::new("create")
                .visible_alias("c")
                .about("Creates a U8 archive")
                .arg_from_usage("[FILE] -f, --file <filename> 'Location of the archive'")
                .arg_from_usage("-v, --verbose 'Make output verbose'")
                .arg_from_usage("[LEVEL] -l, --level <level> 'Sets the compression level between 1 and 10 with 1 being faster but bigger and 10 bing smaller but slower'")
                .arg_from_usage("[EXCLUDE] -e, --exclude... <pattern> 'Exclude files that match the given patterns'")
                .arg(
                    Arg::with_name("INPUT")
                    .help("Addes the given files or directories to the archive that is being created")
                    .long_help("Addes the given files or directories to the archive that is being created.\nExisting archives can also be merged into this one, by prefixing there path with an @ symbol.")
                    .value_name("file")
                    .value_name("directory")
                    .value_name("@archive")
                    .multiple(true)
            )
        )
        .subcommand(
            App::new("list")
            .visible_alias("l")
            .visible_alias("t")
            .about("Lists all entries in a U8 archive")
            .arg_from_usage("[FILE] -f, --file <filename> 'Location of the archive'")
            .arg_from_usage("-v, --verbose 'Make output verbose'")
            .arg_from_usage("[patterns]... 'Only show entries that mach the given patterns'")
            .arg_from_usage("-u, --humansize 'Displays file sizes in a more human-friendly way'")
        )
        .subcommand(
            App::new("extract")
            .visible_alias("e")
            .visible_alias("x")
            .about("Extracts a U8 archive")
            .arg_from_usage("[FILE] -f, --file <filename> 'Location of the archive'")
            .arg_from_usage("-v, --verbose 'Make output verbose'")
            .arg_from_usage("[OUTPUT] -o, --output <directory> 'A directory to output to'")
            .arg_from_usage("[patterns]... 'Only extracts entries that macht the given patterns'")
        )
        .get_matches();

    match run(matches) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    }
}

fn run(matches: ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        //("create", matches) => create::CreateArchive::read(matches.unwrap())?.run()?,
        ("list", matches) => list::ListArchive::read(matches.unwrap())?.run()?,
        ("extract", matches) => extract::ExtractArchive::read(matches.unwrap())?.run()?,
        (cmd, _) => {
            eprintln!("Unknowen subcommand: {}", cmd)
        }
    }
    Ok(())
}
