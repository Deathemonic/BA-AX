use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, fs, process};

use libloading::Library;

use crate::error::{FfiError, FlatError};
use crate::flat;

#[repr(C)]
struct FfiResult {
    json: *mut c_char,
    error: *mut c_char,
    success: c_int
}

type ResolveFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type DumpTableFn = unsafe extern "C" fn(*const c_char, *mut u8, usize) -> FfiResult;
type DumpRowsFn =
    unsafe extern "C" fn(*const c_char, *const *const u8, *const usize, usize) -> FfiResult;
type FreeFn = unsafe extern "C" fn(*mut c_char);
type VersionFn = unsafe extern "C" fn() -> *const c_char;

pub struct Api {
    _library: Library,
    _plugin: PathBuf,
    resolve_table: ResolveFn,
    resolve_row: ResolveFn,
    dump_table: DumpTableFn,
    dump_rows: DumpRowsFn,
    free: FreeFn,
    version: VersionFn
}

static API: OnceLock<Api> = OnceLock::new();
static PLUGIN_ID: AtomicU64 = AtomicU64::new(0);

impl Api {
    unsafe fn load(path: PathBuf) -> Result<Self, FfiError> {
        let library = unsafe { Library::new(&path) }?;

        Ok(Self {
            resolve_table: unsafe { symbol(&library, b"baax_resolve_table\0") }?,
            resolve_row: unsafe { symbol(&library, b"baax_resolve_row\0") }?,
            dump_table: unsafe { symbol(&library, b"baax_dump_table\0") }?,
            dump_rows: unsafe { symbol(&library, b"baax_dump_rows\0") }?,
            free: unsafe { symbol(&library, b"baax_free_string\0") }?,
            version: unsafe { symbol(&library, b"baax_version\0") }?,
            _library: library,
            _plugin: path
        })
    }

    pub fn version(&self) -> Result<String, FfiError> {
        let value = unsafe { (self.version)() };
        if value.is_null() {
            return Err(FfiError::NullResult("version"));
        }

        Ok(unsafe { CStr::from_ptr(value) }.to_str()?.to_owned())
    }

    pub fn resolve_table(&self, filename: &str) -> Result<Option<String>, FfiError> {
        self.resolve(filename, self.resolve_table)
    }

    pub fn resolve_row(&self, table_name: &str) -> Result<Option<String>, FfiError> {
        self.resolve(table_name, self.resolve_row)
    }

    pub fn dump_table(&self, table: &str, bytes: &mut [u8]) -> Result<String, FfiError> {
        let table = CString::new(table)?;
        let result = unsafe { (self.dump_table)(table.as_ptr(), bytes.as_mut_ptr(), bytes.len()) };

        self.result(result)
    }

    pub fn dump_rows(&self, row_type: &str, blobs: &[&[u8]]) -> Result<String, FfiError> {
        let row_type = CString::new(row_type)?;
        let mut pointers = Vec::with_capacity(blobs.len());
        let mut lengths = Vec::with_capacity(blobs.len());

        for blob in blobs {
            pointers.push(blob.as_ptr());
            lengths.push(blob.len());
        }

        let result = unsafe {
            (self.dump_rows)(row_type.as_ptr(), pointers.as_ptr(), lengths.as_ptr(), blobs.len())
        };

        self.result(result)
    }

    fn resolve(&self, value: &str, resolver: ResolveFn) -> Result<Option<String>, FfiError> {
        let value = CString::new(value)?;
        let result = unsafe { resolver(value.as_ptr()) };

        if result.is_null() {
            return Ok(None);
        }

        self.take(result, "resolved type").map(Some)
    }

    fn result(&self, result: FfiResult) -> Result<String, FfiError> {
        if result.success != 0 {
            return self.take(result.json, "JSON");
        }

        Err(FfiError::Plugin(self.take(result.error, "error")?.into_boxed_str()))
    }

    fn take(&self, value: *mut c_char, label: &'static str) -> Result<String, FfiError> {
        if value.is_null() {
            return Err(FfiError::NullResult(label));
        }

        let result = unsafe { CStr::from_ptr(value) }.to_str().map(str::to_owned);
        unsafe { (self.free)(value) };
        Ok(result?)
    }
}

unsafe fn symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T, libloading::Error> {
    Ok(*unsafe { library.get::<T>(name) }?)
}

pub fn api() -> Result<&'static Api, FfiError> { API.get().ok_or(FfiError::NotLoaded) }

pub fn load(path: impl AsRef<Path>) -> Result<(), FfiError> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(FfiError::NotFound(path.to_owned()));
    }

    let plugin = if flat::is_flat(path)? { materialize(path)? } else { path.to_owned() };
    let api = unsafe { Api::load(plugin) }?;
    API.set(api).map_err(|_| FfiError::AlreadyLoaded)
}

fn materialize(path: &Path) -> Result<PathBuf, FfiError> {
    let bytes = flat::extract_host(path)?;
    let id = PLUGIN_ID.fetch_add(1, Ordering::Relaxed);
    let temp = env::temp_dir().join(format!(
        "baax-{}-{}.{}",
        process::id(),
        id,
        flat::lib_ext(flat::HOST_TRIPLE)
    ));
    fs::write(&temp, bytes).map_err(FlatError::from)?;

    #[cfg(unix)]
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o700)).map_err(FlatError::from)?;

    Ok(temp)
}

pub fn version() -> Result<String, FfiError> { api()?.version() }
