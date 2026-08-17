use std::os::raw::{c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{ptr, slice, str};

pub const SINK_VERSION: u32 = 1;

pub const TAG_NULL: u32 = 0;
pub const TAG_BOOL: u32 = 1;
pub const TAG_I8: u32 = 2;
pub const TAG_I16: u32 = 3;
pub const TAG_I32: u32 = 4;
pub const TAG_I64: u32 = 5;
pub const TAG_U8: u32 = 6;
pub const TAG_U16: u32 = 7;
pub const TAG_U32: u32 = 8;
pub const TAG_U64: u32 = 9;
pub const TAG_F32: u32 = 10;
pub const TAG_F64: u32 = 11;
pub const TAG_STR: u32 = 12;
pub const TAG_ENUM: u32 = 13;

pub const SCALAR: i64 = -1;

const FAILED: c_int = 1;
const PASSED: c_int = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Cell {
    pub name: *const u8,
    pub name_len: usize,
    pub text: *const u8,
    pub text_len: usize,
    pub index: i64,
    pub signed: i64,
    pub unsigned: u64,
    pub real: f64,
    pub tag: u32
}

pub type RowFn = unsafe extern "C" fn(*mut c_void) -> c_int;
pub type FieldFn = unsafe extern "C" fn(*mut c_void, *const Cell) -> c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sink {
    pub version: u32,
    pub size: u32,
    pub userdata: *mut c_void,
    pub begin_row: Option<RowFn>,
    pub field: Option<FieldFn>,
    pub end_row: Option<RowFn>
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Null,
    Bool,
    Int,
    Long,
    Float,
    String
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Real(f64),
    Text(&'a str)
}

pub struct Field<'a> {
    pub name: &'a str,
    pub index: i64,
    pub kind: Kind,
    pub value: Value<'a>
}

pub trait Collector {
    fn begin_row(&mut self);

    fn field(&mut self, field: &Field<'_>);

    fn end_row(&mut self);
}

pub fn size() -> u32 { u32::try_from(size_of::<Sink>()).unwrap_or(u32::MAX) }

impl Kind {
    const fn from_tag(tag: u32) -> Self {
        match tag {
            TAG_BOOL => Self::Bool,
            TAG_I8 | TAG_I16 | TAG_I32 | TAG_U8 | TAG_U16 | TAG_U32 => Self::Int,
            TAG_I64 | TAG_U64 => Self::Long,
            TAG_F32 | TAG_F64 => Self::Float,
            TAG_STR | TAG_ENUM => Self::String,
            _ => Self::Null
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Null | Self::String => "string"
        }
    }
}

impl Cell {
    unsafe fn borrow<'a>(text: *const u8, len: usize) -> Option<&'a str> {
        if text.is_null() {
            return None;
        }

        str::from_utf8(unsafe { slice::from_raw_parts(text, len) }).ok()
    }

    unsafe fn value(&self) -> Option<Value<'_>> {
        let value = match self.tag {
            TAG_NULL => Value::Null,
            TAG_BOOL => Value::Bool(self.unsigned != 0),
            TAG_I8 | TAG_I16 | TAG_I32 | TAG_I64 => Value::Signed(self.signed),
            TAG_U8 | TAG_U16 | TAG_U32 | TAG_U64 => Value::Unsigned(self.unsigned),
            TAG_F32 | TAG_F64 => Value::Real(self.real),
            TAG_STR | TAG_ENUM => {
                Value::Text(unsafe { Self::borrow(self.text, self.text_len) }.unwrap_or_default())
            }
            _ => return None
        };

        Some(value)
    }

    unsafe fn field(&self) -> Option<Field<'_>> {
        Some(Field {
            name: unsafe { Self::borrow(self.name, self.name_len) }?,
            index: self.index,
            kind: Kind::from_tag(self.tag),
            value: unsafe { self.value() }?
        })
    }
}

impl Sink {
    pub fn new<C: Collector>(collector: &mut C) -> Self {
        Self {
            version: SINK_VERSION,
            size: size(),
            userdata: ptr::from_mut(collector).cast(),
            begin_row: Some(begin_row::<C>),
            field: Some(field::<C>),
            end_row: Some(end_row::<C>)
        }
    }
}

unsafe fn guard<C, F>(userdata: *mut c_void, run: F) -> c_int
where
    F: FnOnce(&mut C)
{
    let Some(collector) = (unsafe { userdata.cast::<C>().as_mut() }) else {
        return FAILED;
    };

    catch_unwind(AssertUnwindSafe(|| run(collector))).map_or(FAILED, |()| PASSED)
}

unsafe extern "C" fn begin_row<C: Collector>(userdata: *mut c_void) -> c_int {
    unsafe { guard(userdata, C::begin_row) }
}

unsafe extern "C" fn end_row<C: Collector>(userdata: *mut c_void) -> c_int {
    unsafe { guard(userdata, C::end_row) }
}

unsafe extern "C" fn field<C: Collector>(userdata: *mut c_void, cell: *const Cell) -> c_int {
    let Some(cell) = (unsafe { cell.as_ref() }) else {
        return FAILED;
    };
    let Some(field) = (unsafe { cell.field() }) else {
        return FAILED;
    };

    unsafe { guard(userdata, |collector: &mut C| collector.field(&field)) }
}
