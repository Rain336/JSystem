use super::AppSettings;
use atty::Stream;
use bcsv::{DataType, DataValue, Table};
use color_eyre::Result;
use comfy_table::{modifiers, presets, Attribute, Cell, CellAlignment};
use csv::{StringRecord, Writer};
use std::collections::HashMap;

pub fn decode(options: AppSettings) -> Result<()> {
    let crack_map = options.create_crack_map()?;
    let bcsv = options.read_input_bcsv()?;

    let header = create_header_record(&bcsv, &crack_map);

    if let Some(path) = options.output {
        let mut writer = Writer::from_path(path)?;
        writer.write_byte_record(header.as_byte_record())?;

        for row in bcsv.iter() {
            writer.write_byte_record(
                row.iter()
                    .map(|x| x.to_string())
                    .collect::<StringRecord>()
                    .as_byte_record(),
            )?;
        }

        writer.flush()?;
    } else if atty::is(Stream::Stdout) {
        let mut table = comfy_table::Table::new();
        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_SOLID_INNER_BORDERS)
            .set_header(header.iter().collect::<Vec<&str>>());

        for row in &bcsv {
            write_row_terminal(&mut table, row);
        }

        println!("{table}");
    } else {
        let mut writer = Writer::from_writer(std::io::stdout());
        writer.write_byte_record(header.as_byte_record())?;

        for row in bcsv.iter() {
            writer.write_byte_record(
                row.iter()
                    .map(|x| x.to_string())
                    .collect::<StringRecord>()
                    .as_byte_record(),
            )?;
        }

        writer.flush()?;
    }

    Ok(())
}

fn write_row_terminal(table: &mut comfy_table::Table, row: &[DataValue]) {
    table.add_row(row.iter().map(|x| {
        Cell::new(x.to_string())
            .set_alignment(match x.ty() {
                DataType::Int32 => CellAlignment::Right,
                DataType::InlineString => CellAlignment::Left,
                DataType::Float => CellAlignment::Right,
                DataType::UInt32 => CellAlignment::Right,
                DataType::Int16 => CellAlignment::Right,
                DataType::Int8 => CellAlignment::Right,
                DataType::OffsetString => CellAlignment::Left,
                DataType::Null => CellAlignment::Right,
            })
            .add_attribute(match x.ty() {
                DataType::Null => Attribute::Bold,
                _ => Attribute::Reset,
            })
    }));
}

fn create_header_record(bcsv: &Table, crack_map: &HashMap<u32, String>) -> StringRecord {
    let mut header = Vec::with_capacity(bcsv.column_count());
    for definition in bcsv.definitions() {
        header.push(match crack_map.get(&definition.name) {
            Some(x) => format!("{}({})", x, definition.ty),
            None => format!("{:#X}({})", definition.name, definition.ty),
        });
    }
    header.into()
}
