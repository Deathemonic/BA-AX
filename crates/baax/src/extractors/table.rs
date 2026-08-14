use std::io::{Cursor, Read};

use bacy::hash::xxhash;
use bacy::math::mersenne::MersenneTwister;
use base64::Engine;
use base64::engine::general_purpose;
use zip::ZipArchive;

use crate::error::TableZipError;

pub struct TableZipFile {
    archive: ZipArchive<Cursor<Vec<u8>>>,
    password: [u8; 20]
}

impl TableZipFile {
    pub fn new(buf: Vec<u8>, filename: impl AsRef<[u8]>) -> Result<Self, TableZipError> {
        let hash = xxhash::calculate_hash(filename.as_ref()) as u32;
        let mut rng = MersenneTwister::new(hash);

        let mut next_buf = [0u8; 15];
        rng.next_bytes(&mut next_buf);

        let mut password = [0u8; 20];
        general_purpose::STANDARD.encode_slice(next_buf, &mut password)?;

        let archive = ZipArchive::new(Cursor::new(buf))?;
        Ok(Self { archive, password })
    }

    pub fn get_by_name(&mut self, name: &str) -> Result<Vec<u8>, TableZipError> {
        let mut file = self.archive.by_name_decrypt(name, &self.password)?;
        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn extract_all(&mut self) -> Result<Vec<(String, Vec<u8>)>, TableZipError> {
        let mut files = Vec::with_capacity(self.archive.len());
        for i in 0..self.archive.len() {
            let mut file = self.archive.by_index_decrypt(i, &self.password)?;
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)?;
            files.push((file.name().to_string(), buf));
        }
        Ok(files)
    }
}
