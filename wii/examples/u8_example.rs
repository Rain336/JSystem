use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use wii::FileFormat;
use wii::u8::U8Archive;

fn main() -> Result<(), Box<dyn Error>> {
    let mut archive_path = std::env::current_dir().unwrap();
    archive_path.push("wii");
    archive_path.push("examples");
    archive_path.push("opening.u8");
    println!("The archive is at: {}", archive_path.display());
    let mut file = BufReader::new(File::open(&archive_path)?);
    let archive = U8Archive::read(&mut file)?;
    for (path, data) in archive.iter() {
        println!("{} - {}", path, data.map(|x| x.len()).unwrap_or_default());
    }
    Ok(())
}