use std::slice;

use rust_xlsxwriter::{Workbook, Worksheet};
use serde_json::{Map, Value};

use crate::error::ExtractError;

const SHEET_NAME_LIMIT: usize = 31;
const CELL_LIMIT: usize = 32_767;
const TYPE_ROW: u32 = 0;
const KEY_ROW: u32 = 1;

type Row<'a> = &'a Map<String, Value>;

struct Column<'a> {
    key: &'a str,
    kind: &'static str,
    element: Option<usize>
}

impl<'a> Column<'a> {
    fn value<'b>(&self, row: Row<'b>) -> Option<&'b Value> {
        let value = row.get(self.key)?;

        match self.element {
            Some(element) => value.as_array()?.get(element),
            None => Some(value)
        }
    }
}

pub fn to_xlsx(json: &str, name: &str) -> Result<Vec<u8>, ExtractError> {
    let value: Value = serde_json::from_str(json)?;
    let rows = rows(&value);

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(clip(name, SHEET_NAME_LIMIT))?;

    write_columns(worksheet, &rows)?;

    Ok(workbook.save_to_buffer()?)
}

fn clip(value: &str, limit: usize) -> &str {
    match value.char_indices().nth(limit) {
        Some((index, _)) => &value[..index],
        None => value
    }
}

fn sanitize(value: &Value) -> &Value {
    let Some(map) = value.as_object() else {
        return value;
    };

    if map.len() != 1 {
        return value;
    }

    map.values().next().filter(|inner| inner.is_array()).unwrap_or(value)
}

fn rows(value: &Value) -> Vec<Row<'_>> {
    match sanitize(value) {
        Value::Array(items) => items.iter().filter_map(Value::as_object).collect(),
        Value::Object(map) => vec![map],
        _ => Vec::new()
    }
}

fn keys<'a>(rows: &[Row<'a>]) -> Vec<&'a str> {
    let mut keys: Vec<&str> = Vec::new();

    for row in rows {
        for key in row.keys() {
            if !keys.contains(&key.as_str()) {
                keys.push(key);
            }
        }
    }

    keys
}

fn columns<'a>(rows: &[Row<'a>]) -> Vec<Column<'a>> {
    let mut columns = Vec::new();

    for key in keys(rows) {
        let kind = column_type(rows, key);

        match width(rows, key) {
            Some(width) => columns.extend((0..width).map(|element| Column {
                key,
                kind,
                element: Some(element)
            })),
            None => columns.push(Column { key, kind, element: None })
        }
    }

    columns
}

fn values<'a, 'b>(rows: &'b [Row<'a>], key: &'b str) -> impl Iterator<Item = &'a Value> + 'b {
    rows.iter().copied().filter_map(|row| row.get(key))
}

fn width(rows: &[Row<'_>], key: &str) -> Option<usize> {
    values(rows, key).filter_map(Value::as_array).map(Vec::len).max().map(|len| len.max(1))
}

fn scalars(value: &Value) -> &[Value] {
    match value {
        Value::Array(items) => items,
        scalar => slice::from_ref(scalar)
    }
}

fn column_type(rows: &[Row<'_>], key: &str) -> &'static str {
    values(rows, key).flat_map(scalars).find(|value| !value.is_null()).map_or("string", value_type)
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_f64() => "float",
        Value::Number(number) => number.as_i64().map_or("long", integer_type),
        _ => "string"
    }
}

fn integer_type(value: i64) -> &'static str {
    if i32::try_from(value).is_ok() { "int" } else { "long" }
}

fn write_columns(worksheet: &mut Worksheet, rows: &[Row<'_>]) -> Result<(), ExtractError> {
    for (index, column) in columns(rows).into_iter().enumerate() {
        let col = u16::try_from(index).map_err(|_| ExtractError::InvalidFormat)?;

        worksheet.write_string(TYPE_ROW, col, column.kind)?;
        worksheet.write_string(KEY_ROW, col, column.key)?;

        for (offset, row) in rows.iter().enumerate() {
            let Some(value) = column.value(row) else {
                continue;
            };

            let index = u32::try_from(offset).map_err(|_| ExtractError::InvalidFormat)?;
            write_value(worksheet, index + KEY_ROW + 1, col, value)?;
        }
    }

    Ok(())
}

fn write_value(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    value: &Value
) -> Result<(), ExtractError> {
    match value {
        Value::Null => return Ok(()),
        Value::Bool(flag) => worksheet.write_boolean(row, col, *flag)?,
        Value::Number(number) => {
            worksheet.write_number(row, col, number.as_f64().unwrap_or_default())?
        }
        Value::String(text) => worksheet.write_string(row, col, clip(text, CELL_LIMIT))?,
        other => worksheet.write_string(row, col, clip(&other.to_string(), CELL_LIMIT))?
    };

    Ok(())
}
