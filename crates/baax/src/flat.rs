use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::FlatError;

const MAGIC: &[u8; 4] = b"FLAT";
const VERSION: u16 = 1;
const TRIPLE_LEN: usize = 32;
const ENTRY_SIZE: usize = 60;

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
    let mut file = File::open(path)?;
    let entries = read_header(&mut file)?;
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

    file.seek(SeekFrom::Start(entry.offset))?;
    let mut compressed =
        vec![0u8; usize::try_from(entry.compressed_size).map_err(|_| FlatError::Invalid)?];
    file.read_exact(&mut compressed)?;

    let raw = zstd::decode_all(compressed.as_slice()).map_err(|_| FlatError::Decompress)?;
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

fn read_header<R: Read>(reader: &mut R) -> Result<Vec<Entry>, FlatError> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(FlatError::Invalid);
    }

    let mut short = [0u8; 2];
    reader.read_exact(&mut short)?;
    let version = u16::from_le_bytes(short);
    if version != VERSION {
        return Err(FlatError::UnsupportedVersion(version));
    }

    reader.read_exact(&mut short)?;
    let count = u16::from_le_bytes(short);
    if count == 0 {
        return Err(FlatError::Invalid);
    }

    let mut entries = Vec::with_capacity(count as usize);
    let mut buf = [0u8; ENTRY_SIZE];
    for _ in 0..count {
        reader.read_exact(&mut buf)?;
        entries.push(parse_entry(&buf)?);
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
