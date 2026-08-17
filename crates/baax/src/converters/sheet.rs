use std::collections::HashMap;
use std::sync::Arc;

use baax_plugin::sink::{Collector, Field, Kind, Value};

pub enum Cell {
    Null,
    Bool(bool),
    Number(f64),
    Text(String)
}

pub struct Column {
    pub name: Arc<str>,
    pub index: i64,
    pub kind: Kind
}

#[derive(Default)]
pub struct Sheet {
    columns: Vec<Column>,
    lookup: HashMap<Arc<str>, HashMap<i64, usize>>,
    rows: Vec<Vec<Cell>>,
    pending: Vec<(usize, Cell)>
}

impl Cell {
    fn new(value: &Value<'_>) -> Self {
        match *value {
            Value::Null => Self::Null,
            Value::Bool(flag) => Self::Bool(flag),
            Value::Signed(number) => Self::Number(number as f64),
            Value::Unsigned(number) => Self::Number(number as f64),
            Value::Real(number) => Self::Number(number),
            Value::Text(text) => Self::Text(text.to_owned())
        }
    }
}

impl Sheet {
    pub fn new() -> Self { Self::default() }

    pub fn columns(&self) -> &[Column] { &self.columns }

    pub fn rows(&self) -> &[Vec<Cell>] { &self.rows }

    pub const fn is_empty(&self) -> bool { self.rows.is_empty() }

    fn column(&mut self, field: &Field<'_>) -> usize {
        let existing =
            self.lookup.get(field.name).and_then(|slots| slots.get(&field.index)).copied();

        if let Some(id) = existing {
            self.widen(id, field.kind);
            return id;
        }

        self.insert(field)
    }

    fn insert(&mut self, field: &Field<'_>) -> usize {
        let id = self.columns.len();
        let name = Arc::<str>::from(field.name);

        self.columns.push(Column {
            name: Arc::clone(&name),
            index: field.index,
            kind: field.kind
        });
        self.lookup.entry(name).or_default().insert(field.index, id);

        id
    }

    fn widen(&mut self, id: usize, kind: Kind) {
        if kind == Kind::Null {
            return;
        }

        if let Some(column) = self.columns.get_mut(id)
            && column.kind == Kind::Null
        {
            column.kind = kind;
        }
    }
}

impl Collector for Sheet {
    fn begin_row(&mut self) { self.pending.clear(); }

    fn field(&mut self, field: &Field<'_>) {
        let id = self.column(field);
        self.pending.push((id, Cell::new(&field.value)));
    }

    fn end_row(&mut self) {
        let mut row = Vec::new();
        row.resize_with(self.columns.len(), || Cell::Null);

        for (id, cell) in self.pending.drain(..) {
            if let Some(slot) = row.get_mut(id) {
                *slot = cell;
            }
        }

        self.rows.push(row);
    }
}
