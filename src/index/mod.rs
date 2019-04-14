use std::fs::File;
use std::rc::Rc;
use crate::utils;
use log::{info, debug, error};
use std::io::{ErrorKind, Error};

const CaFormatIndex: u64 = 0x96824d9c7b129ff9;
const CaFormatTable: u64 = 0xe75b9e112f17417d;
const CaFormatSHA512256: u64 = 0x2000000000000000;
const CaFormatTableTailMarker: u64 = 0x4b4f050e5549ecd1;
const FeatureFlags: u64 = 11529215046068469760;

pub struct LocalIndexFile {
    pub path: String,
    pub file: Rc<File>,
    pub chunk_table_size: u64,
    pub chunk_data: Vec<ChunkData>
}

impl LocalIndexFile {
    pub fn new(path: &str) -> LocalIndexFile {
        match File::create(path) {
            Ok(f) => {
                LocalIndexFile {
                    path: String::from(path),
                    file: Rc::new(f),
                    chunk_table_size: 0,
                    chunk_data: Vec::new()
                }
            },
            Err(e) => {
                panic!("Could not open index file, {:?}",e);
            }
        }
    }
    pub fn open(path: &str) -> LocalIndexFile {
        match File::open(path) {
            Ok(f) => {
                LocalIndexFile {
                    path: String::from(path),
                    file: Rc::new(f),
                    chunk_table_size: 0,
                    chunk_data: Vec::new()
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
    //TODO: rename to load
    fn read(&mut self);
    fn getChunkData(&self) -> Vec<ChunkData>;
}

impl Index for LocalIndexFile {
    fn write_header(&mut self, min: u64, max: u64, avg: u64) {
        info!("Started writing to index file");
        let size = 48;
        let mut file = Rc::get_mut(&mut self.file).unwrap();
        utils::write_u64(file, size).unwrap();
        utils::write_u64(file, CaFormatIndex).unwrap();
        utils::write_u64(file, FeatureFlags).unwrap();
        utils::write_u64(file, min).unwrap();
        utils::write_u64(file, avg).unwrap();
        utils::write_u64(file, max).unwrap();
        // Header for chunks
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
        utils::write_u64(file, 0).unwrap();
        utils::write_u64(file, 0).unwrap();
        utils::write_u64(file, 48).unwrap();
        utils::write_u64(file, self.chunk_table_size + 40).unwrap();
        utils::write_u64(file, CaFormatTableTailMarker).unwrap();
        self.chunk_table_size += (5 * 8);
        debug!("Wrote tail marker to index file");
        info!("Finished writing to index file");
    }

    fn read(&mut self) {
        // read file
        let mut f = Rc::get_mut(&mut self.file).unwrap();
        let _headerSize = utils::read_u64(&mut f);
        let headertype = utils::read_u64(&mut f);
        let mut chunkItems: Vec<ChunkData> = Vec::new();
        if headertype == CaFormatIndex {
            // Reading index file
            info!("Found index file");
            let indexFeatureFlags = utils::read_u64(&mut f);
            let indexChunkSizeMin = utils::read_u64(&mut f);
            let indexChunkSizeAvg = utils::read_u64(&mut f);
            let indexChunkSizeMax = utils::read_u64(&mut f);

            if (indexFeatureFlags & CaFormatSHA512256) as u64 == 0 {
                panic!("Only supports SHA 512 / 256")
            }

            // Reading chunk table
            let headerSize = utils::read_u64(&mut f);
            let headertype = utils::read_u64(&mut f);
            if headerSize != std::u64::MAX {
                panic!("Invalid size");
            }
            if headertype == CaFormatTable {
                let mut tableItems: Vec<TableItem> = Vec::new();
                loop {
                    let offset = utils::read_u64(&mut f);
                    if offset == 0 {
                        break;
                    }
                    let mut chunk_id: [u8;32] = [0;32];
                    match utils::read_32_bytes(&mut f, &mut chunk_id) {
                        Ok(()) => {
                            tableItems.push(TableItem{
                                offset: offset,
                                id: chunk_id 
                            })
                        },
                        Err(e) => {
                            if e.kind() == ErrorKind::UnexpectedEof {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error while reading chunk id {}",e)
                        }
                    }
                }
                debug!("Number of chunks found {}", tableItems.len());
                let tailMarker1 = utils::read_u64(&mut f);
                if tailMarker1 != 0 {
                    panic!("tail marker 1 not found");
                }
                utils::read_u64(&mut f);// Read index offset
                utils::read_u64(&mut f);// size
                let tailMarker2 = utils::read_u64(&mut f);
                if tailMarker2 != CaFormatTableTailMarker {
                    panic!("tail marker 2 is not found")
                }

                // Reversing and putting chunks in proper order
                let mut lastOffset: u64 = 0;
                for c in tableItems.iter() {
                    let size = c.offset - lastOffset;
                    debug!("chunk start {}, size {} and id {:?}", lastOffset, size, c.id);   
                    chunkItems.push(ChunkData{
                        id: c.id,
                        start: lastOffset,
                        size: size
                    });
                    lastOffset = c.offset;
                    self.chunk_table_size += 1;
                }
                self.chunk_data = chunkItems;
            } else {
                error!("Invalid chunk table found inside index")
            }
        } else {
            error!("Not an index file");
        }
    }
    fn getChunkData(&self) -> Vec<ChunkData> {
        self.chunk_data.clone()
    }
}

use std::clone::Clone;

// TableItem ---------------------------------------------------------------
pub struct TableItem {
    offset: u64,
    id: [u8;32]
}

#[derive(Clone)]
pub struct ChunkData {
    pub id: [u8;32],
    pub start: u64,
    pub size: u64
}