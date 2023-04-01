use super::AppSettings;
use bcsv::byteorder::{BigEndian, LittleEndian};
use bcsv::{DataType, DataValue, Table};
use color_eyre::Result;

pub fn encode(options: AppSettings) -> Result<()> {
    let (headers, rows) = options.read_input_csv()?;

    let mut bcsv = Table::new();
    for header in headers.into_iter() {
        let (hash, ty) = find_data_type(header);
        bcsv.push_column(hash, ty, &ty.default_value())?;
    }

    for (i, row) in rows.iter().enumerate() {
        if row.len() != bcsv.column_count() {
            eprintln!("Couldn't map row {i}, since it has an unmatching amout of columns.");
            continue;
        }

        let mut mapped = bcsv
            .definitions()
            .iter()
            .zip(row.into_iter())
            .enumerate()
            .map(|(j, (definition, data))| match definition.ty {
                DataType::Int32 => match data.parse::<i32>() {
                    Ok(x) => DataValue::Int32(x),
                    Err(e) => {
                        eprintln!("Couldn't map value in row {i} column {j}: {e}");
                        DataValue::Int32(0)
                    }
                },
                DataType::InlineString => {
                    eprintln!("Inline string converted to offset string in row {i} column {j}");
                    DataValue::OffsetString(data.into())
                }
                DataType::Float => match data.parse::<f32>() {
                    Ok(x) => DataValue::Float(x),
                    Err(e) => {
                        eprintln!("Couldn't map value in row {i} column {j}: {e}");
                        DataValue::Float(0.0)
                    }
                },
                DataType::UInt32 => match data.parse::<u32>() {
                    Ok(x) => DataValue::UInt32(x),
                    Err(e) => {
                        eprintln!("Couldn't map value in row {i} column {j}: {e}");
                        DataValue::UInt32(0)
                    }
                },
                DataType::Int16 => match data.parse::<i16>() {
                    Ok(x) => DataValue::Int16(x),
                    Err(e) => {
                        eprintln!("Couldn't map value in row {i} column {j}: {e}");
                        DataValue::Int16(0)
                    }
                },
                DataType::Int8 => match data.parse::<i8>() {
                    Ok(x) => DataValue::Int8(x),
                    Err(e) => {
                        eprintln!("Couldn't map value in row {i} column {j}: {e}");
                        DataValue::Int8(0)
                    }
                },
                DataType::OffsetString => DataValue::OffsetString(data.into()),
                DataType::Null => DataValue::Null,
            })
            .collect();

        bcsv.push_row(&mut mapped)?;
    }

    match options.output {
        Some(x) => {
            if options.little_endian {
                bcsv.save::<LittleEndian>(x)?;
            } else {
                bcsv.save::<BigEndian>(x)?;
            }
        }
        None => {
            let writer = std::io::stdout().lock();

            if options.little_endian {
                bcsv.write::<LittleEndian>(writer)?;
            } else {
                bcsv.write::<BigEndian>(writer)?;
            }
        }
    }

    Ok(())
}

fn find_data_type(header: &str) -> (u32, DataType) {
    let (ty, header) = if header.as_bytes().last() == Some(&b')') {
        match header.rfind('(') {
            Some(idx) => (
                match &header[(idx + 1)..(header.len() - 1)] {
                    "Long" => DataType::Int32,
                    "Float" => DataType::Float,
                    "Long2" => DataType::UInt32,
                    "Short" => DataType::Int16,
                    "Char" => DataType::Int8,
                    "Null" => DataType::Null,
                    _ => DataType::OffsetString,
                },
                &header[..idx],
            ),
            None => (DataType::OffsetString, header),
        }
    } else {
        (DataType::OffsetString, header)
    };

    let hash = parse_number_or_hash(header);

    (hash, ty)
}

fn parse_number_or_hash(value: &str) -> u32 {
    match &value[..2] {
        "0x" | "0X" => match u32::from_str_radix(&value[2..], 16) {
            Ok(x) => x,
            Err(_) => bcsv::jgadget_hash(value.as_bytes()),
        },
        _ => match value.parse::<u32>() {
            Ok(x) => x,
            Err(_) => bcsv::jgadget_hash(value.as_bytes()),
        },
    }
}
