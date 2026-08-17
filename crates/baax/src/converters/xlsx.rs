use rust_xlsxwriter::{Workbook, Worksheet};

use crate::converters::sheet::{Cell, Sheet};
use crate::error::ExtractError;

const SHEET_NAME_LIMIT: usize = 31;
const CELL_LIMIT: usize = 32_767;
const TYPE_ROW: u32 = 0;
const KEY_ROW: u32 = 1;

pub fn to_xlsx(sheet: &Sheet, name: &str) -> Result<Vec<u8>, ExtractError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(clip(name, SHEET_NAME_LIMIT))?;

    write_columns(worksheet, sheet)?;

    Ok(workbook.save_to_buffer()?)
}

fn clip(value: &str, limit: usize) -> &str {
    match value.char_indices().nth(limit) {
        Some((index, _)) => &value[..index],
        None => value
    }
}

fn write_columns(worksheet: &mut Worksheet, sheet: &Sheet) -> Result<(), ExtractError> {
    for (index, column) in sheet.columns().iter().enumerate() {
        let col = u16::try_from(index).map_err(|_| ExtractError::InvalidFormat)?;

        worksheet.write_string(TYPE_ROW, col, column.kind.label())?;
        worksheet.write_string(KEY_ROW, col, &*column.name)?;
    }

    for (offset, row) in sheet.rows().iter().enumerate() {
        let index = u32::try_from(offset).map_err(|_| ExtractError::InvalidFormat)?;

        for (position, cell) in row.iter().enumerate() {
            let col = u16::try_from(position).map_err(|_| ExtractError::InvalidFormat)?;
            write_cell(worksheet, index + KEY_ROW + 1, col, cell)?;
        }
    }

    Ok(())
}

fn write_cell(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    cell: &Cell
) -> Result<(), ExtractError> {
    match cell {
        Cell::Null => return Ok(()),
        Cell::Bool(flag) => worksheet.write_boolean(row, col, *flag)?,
        Cell::Number(number) => worksheet.write_number(row, col, *number)?,
        Cell::Text(text) => worksheet.write_string(row, col, clip(text, CELL_LIMIT))?
    };

    Ok(())
}
