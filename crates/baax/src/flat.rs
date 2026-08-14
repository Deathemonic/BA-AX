use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use memmap2::Mmap;

use crate::error::FlatError;

const MAGIC: &[u8; 4] = b"FLAT";
const VERSION: u16 = 1;
const TRIPLE_LEN: usize = 32;
const ENTRY_SIZE: usize = 60;
const HEADER_FIXED_LEN: usize = 8;

pub const HOST_TRIPLE: &str = env!("BAAX_TARGET");

struct Entry {
    target_triple: String,
    offset: u64,
    compressed_size: u64,
    uncompressed_size: u64,
    checksum: u32
}

pub fn is_flat(path: &Path) -> Result<bool, FlatError> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    match file.read_exact(&mut magic) {
        Ok(()) => Ok(&magic == MAGIC),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error.into())
    }
}

pub fn extract_host(path: &Path) -> Result<Vec<u8>, FlatError> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let entries = read_header(&mmap)?;
    let entry =
        entries.iter().find(|entry| entry.target_triple == HOST_TRIPLE).ok_or_else(|| {
            FlatError::TripleNotFound {
                host: HOST_TRIPLE.into(),
                available: entries
                    .iter()
                    .map(|entry| entry.target_triple.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into_boxed_str()
            }
        })?;

    let start = usize::try_from(entry.offset).map_err(|_| FlatError::Invalid)?;
    let comp_len = usize::try_from(entry.compressed_size).map_err(|_| FlatError::Invalid)?;
    let end = start.checked_add(comp_len).ok_or(FlatError::Invalid)?;
    let compressed = mmap.get(start..end).ok_or(FlatError::Invalid)?;

    let raw = zstd::decode_all(compressed).map_err(|_| FlatError::Decompress)?;
    if raw.len() as u64 != entry.uncompressed_size {
        return Err(FlatError::Invalid);
    }
    if crc32fast::hash(&raw) != entry.checksum {
        return Err(FlatError::ChecksumMismatch);
    }

    Ok(raw)
}

pub fn lib_ext(triple: &str) -> &'static str {
    if triple.contains("windows") {
        "dll"
    } else if triple.contains("apple") {
        "dylib"
    } else {
        "so"
    }
}

fn read_header(mmap: &Mmap) -> Result<Vec<Entry>, FlatError> {
    if mmap.len() < HEADER_FIXED_LEN {
        return Err(FlatError::Invalid);
    }

    let magic: [u8; 4] = mmap[0..4].try_into().map_err(|_| FlatError::Invalid)?;
    if &magic != MAGIC {
        return Err(FlatError::Invalid);
    }

    let version = u16::from_le_bytes(mmap[4..6].try_into().map_err(|_| FlatError::Invalid)?);
    if version != VERSION {
        return Err(FlatError::UnsupportedVersion(version));
    }

    let count = u16::from_le_bytes(mmap[6..8].try_into().map_err(|_| FlatError::Invalid)?);
    if count == 0 {
        return Err(FlatError::Invalid);
    }

    let count = count as usize;
    let table_len = count.checked_mul(ENTRY_SIZE).ok_or(FlatError::Invalid)?;
    let table_end = HEADER_FIXED_LEN.checked_add(table_len).ok_or(FlatError::Invalid)?;
    let table = mmap.get(HEADER_FIXED_LEN..table_end).ok_or(FlatError::Invalid)?;

    let mut entries = Vec::with_capacity(count);
    for chunk in table.chunks_exact(ENTRY_SIZE) {
        let buf: &[u8; ENTRY_SIZE] = chunk.try_into().map_err(|_| FlatError::Invalid)?;
        entries.push(parse_entry(buf)?);
    }

    Ok(entries)
}

fn parse_entry(buf: &[u8; ENTRY_SIZE]) -> Result<Entry, FlatError> {
    let triple: [u8; TRIPLE_LEN] = buf[..TRIPLE_LEN].try_into().map_err(|_| FlatError::Invalid)?;
    let end = triple.iter().position(|&b| b == 0).unwrap_or(TRIPLE_LEN);
    if triple[end..].iter().any(|&b| b != 0) {
        return Err(FlatError::Invalid);
    }
    let target_triple = std::str::from_utf8(&triple[..end])?.to_owned();
    if target_triple.is_empty() {
        return Err(FlatError::Invalid);
    }

    Ok(Entry {
        target_triple,
        offset: u64::from_le_bytes(buf[32..40].try_into().map_err(|_| FlatError::Invalid)?),
        compressed_size: u64::from_le_bytes(
            buf[40..48].try_into().map_err(|_| FlatError::Invalid)?
        ),
        uncompressed_size: u64::from_le_bytes(
            buf[48..56].try_into().map_err(|_| FlatError::Invalid)?
        ),
        checksum: u32::from_le_bytes(buf[56..60].try_into().map_err(|_| FlatError::Invalid)?)
    })
}
