use std::fs::File;
use std::rc::Rc;
use crate::utils;
use log::{info, debug};

pub struct LocalIndexFile {
    pub path: String,
    pub file: Rc<File>,
    pub chunk_table_size: u64
}

impl LocalIndexFile {
    pub fn new(path: &str) -> LocalIndexFile {
        match File::create(path) {
            Ok(f) => {
                LocalIndexFile {
                    path: String::from(path),
                    file: Rc::new(f),
                    chunk_table_size: 0
                }
            },
            Err(e) => {
                panic!("Could not open index file, {:?}",e);
            }
        }
    }
}

pub trait Index {
    // Load index file for extract
    // Add new index entry
    fn write_header(&mut self, min: u64, max: u64, avg: u64);
    fn add_entry(&mut self, start: u64, chunk_id: [u8;32]);
    fn write_tail(&mut self);
}

impl Index for LocalIndexFile {
    fn write_header(&mut self, min: u64, max: u64, avg: u64) {
        info!("Started writing to index file");
        let size = 48;
        let CaFormatIndex = 0x96824d9c7b129ff9;
        let FeatureFlags = 11529215046068469760;
        let mut file = Rc::get_mut(&mut self.file).unwrap();
        utils::write_u64(file, size).unwrap();
        utils::write_u64(file, CaFormatIndex).unwrap();
        utils::write_u64(file, FeatureFlags).unwrap();
        utils::write_u64(file, min).unwrap();
        utils::write_u64(file, avg).unwrap();
        utils::write_u64(file, max).unwrap();
        // Header for chunks
        let CaFormatTable = 0xe75b9e112f17417d;
        utils::write_u64(file, std::u64::MAX);
        self.chunk_table_size += 8;
        utils::write_u64(file, CaFormatTable);
        self.chunk_table_size += 8;
        debug!("Wrote header to index file");
    }
    fn add_entry(&mut self, start: u64, chunk_id: [u8;32]) {
        let mut file = Rc::get_mut(&mut self.file).unwrap();
        utils::write_u64(file, start).unwrap();
        self.chunk_table_size += 8;
        utils::write_32_bytes(file, chunk_id).unwrap();
        self.chunk_table_size += 32;
        debug!("Added chunk entry to index file");
    }
    fn write_tail(&mut self) {
        let mut file = Rc::get_mut(&mut self.file).unwrap();
        let CaFormatTableTailMarker = 0x4b4f050e5549ecd1;
        utils::write_u64(file, 0).unwrap();
        utils::write_u64(file, 0).unwrap();
        utils::write_u64(file, 48).unwrap();
        utils::write_u64(file, self.chunk_table_size + 40).unwrap();
        utils::write_u64(file, CaFormatTableTailMarker).unwrap();
        self.chunk_table_size += (5 * 8);
        debug!("Wrote tail marker to index file");
        info!("Finished writing to index file");
    }
}