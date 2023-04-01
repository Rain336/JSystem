use humansize::{file_size_opts, FileSize};
use jsystem::rarc;
use std::fs;

fn main() {
    let mut archive_path = std::env::current_dir().unwrap();
    archive_path.push("jsystem");
    archive_path.push("examples");
    archive_path.push("EggStarGalaxyScenario.arc");
    println!("The archive is at: {}", archive_path.display());

    let file = fs::File::open(&archive_path).unwrap();
    let archive = rarc::RarcArchive::read(file).unwrap();

    for (path, data) in archive.iter() {
        let size = match data {
            Some(x) => x.len().file_size(file_size_opts::CONVENTIONAL).unwrap(),
            None => "Directory".into(),
        };
        println!("{} {}", path, size);
    }
}
