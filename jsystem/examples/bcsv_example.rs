use jsystem::bcsv::BcsvTable;

fn main() {
    let mut table_path = std::env::current_dir().unwrap();
    table_path.push("jsystem");
    table_path.push("examples");
    table_path.push("scenariodata.bcsv");
    println!("The table is at: {}", table_path.display());
    let buffer = std::fs::read(table_path).unwrap();
    let table = BcsvTable::read(&buffer).unwrap();
    for (hash, ty) in table.definitions() {
        print!("{}({}),", hash, ty);
    }
    println!();
    for row in table.iter() {
        for data in row {
            print!("{},", data);
        }
        println!();
    }
}
