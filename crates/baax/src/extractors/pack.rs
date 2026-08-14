use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use baad_utils::{debug, info};
use bacy::crypto::aes;
use bacy::hash::sha;
use memmap2::Mmap;
use serde::Deserialize;
use tokio::fs;

use crate::error::ExtractError;

const MAGIC: &[u8; 2] = b"4t";
const VERSION: u8 = 0x01;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CatalogEntry {
    offset: i64,
    length: i64
}

pub struct PackFile {
    mmap: Mmap,
    index: HashMap<String, CatalogEntry>
}

impl PackFile {
    pub fn open(path: &Path) -> Result<Self, ExtractError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 22 {
            return Err(ExtractError::InvalidFormat);
        }

        if &mmap[0..2] != MAGIC {
            return Err(ExtractError::InvalidFormat);
        }

        if mmap[4] != VERSION {
            return Err(ExtractError::InvalidFormat);
        }

        let iv: [u8; 16] = mmap[5..21].try_into().map_err(|_| ExtractError::InvalidFormat)?;

        let filename = path
            .file_name()
            .ok_or(ExtractError::FileName)?
            .to_str()
            .ok_or(ExtractError::FromString)?;

        let key = sha::compute_str(filename);

        let len = mmap.len();
        let json_len = i32::from_le_bytes(
            mmap[len - 4..len].try_into().map_err(|_| ExtractError::InvalidFormat)?
        ) as usize;

        let catalog_start = len - 4 - json_len;
        let encrypted = &mmap[catalog_start..len - 4];

        let decrypted = aes::decrypt(encrypted, &key, &iv).map_err(|_| ExtractError::Crypto)?;

        let index: HashMap<String, CatalogEntry> =
            serde_json::from_slice(&decrypted).map_err(|_| ExtractError::InvalidFormat)?;

        Ok(Self { mmap, index })
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.index.iter().map(|(path, entry)| {
            let start = entry.offset as usize;
            let end = start + entry.length as usize;
            (path.as_str(), &self.mmap[start..end])
        })
    }

    pub fn len(&self) -> usize { self.index.len() }

    pub fn is_empty(&self) -> bool { self.index.is_empty() }
}

pub async fn extract_pack(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>
) -> Result<(), ExtractError> {
    let path = path.as_ref();
    let filename =
        path.file_name().ok_or(ExtractError::FileName)?.to_str().ok_or(ExtractError::FromString)?;

    let pack = PackFile::open(path)?;
    let dir = output.as_ref().join(filename.trim_end_matches(".molru"));

    debug!(from = filename, to = %dir.display(), "Extracting");

    fs::create_dir_all(&dir).await?;

    for (name, data) in pack.entries() {
        let out = dir.join(name);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(out, data).await?;
    }

    info!(success = true, filename, "Extracted");
    Ok(())
}

pub async fn extract_all_packs(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>
) -> Result<(), ExtractError> {
    for entry in input.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("molru") {
            extract_pack(&path, &output).await?;
        }
    }

    Ok(())
}