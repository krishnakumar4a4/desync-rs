use std::fs::{DirBuilder, File};
use crypto::sha2::Sha512Trunc256;
use crypto::digest::Digest;
use std::path::{Path, PathBuf};
use zstd::Encoder;
use std::io;
use std::io::Error;


// StoreStats Store the stats for current store
pub struct StoreStats {
    count: u64,
    min_size: u64,
    max_size: u64,
    avg_size: u64,
    processed_bytes: u64,
    new_chunks_count: u64
}
impl StoreStats {
    pub fn new(min: u64, max: u64, avg: u64) -> StoreStats {
        StoreStats {
            count: 0,
            min_size: min,
            max_size: max,
            avg_size: avg,
            processed_bytes: 0,
            new_chunks_count: 0
        }
    }
    pub fn add_item(&mut self, processed_bytes: u64) {
        self.processed_bytes += processed_bytes;
        self.count += 1;
    }
    pub fn add_new_item(&mut self, processed_bytes: u64) {
        self.processed_bytes += processed_bytes;
        self.count += 1;
        self.new_chunks_count += 1;
    }    
}

// LocalStore 
pub struct LocalStore {
    pub path: String,
    pub stats: StoreStats
}

pub trait Store {
    fn create(&self, path: &str) -> PathBuf;
    fn write_item(&mut self, bytes: Vec<u8>) -> [u8;32];
}

impl Store for LocalStore {
    fn create(&self, path: &str) -> PathBuf {
        let mut base_path = Path::new(&self.path);
        let final_path = base_path.join(path);
        match DirBuilder::new().recursive(true).create(final_path.as_path()) {
            Ok(()) => {
                //Log store creation
                final_path
            },
            Err(e) => {
                panic!("Could not create store {:?}", e);
            }
        }
    }

    fn write_item(&mut self, bytes: Vec<u8>) -> [u8;32] {
        // Write to file 
        let mut hasher = Sha512Trunc256::new();
        hasher.input(&bytes);
        let hash_value = hasher.result_str();
        let (sub_dir_name,_) = hash_value.split_at(4);
        let mut chunk_folder = self.create(sub_dir_name);
        chunk_folder.push(hash_value);
        chunk_folder.set_extension("cacnk");
 
        if chunk_folder.exists() {
            self.stats.add_item(bytes.len() as u64);
        } else {
            let new_chunk_file = create_chunk_file(chunk_folder);
            match new_chunk_file {
                Ok(f) => {
                    let mut encoder = Encoder::new(f,21).unwrap();
                    io::copy(&mut bytes.as_slice(), &mut encoder).expect("Error: Cannot write compressed data to file");
                    encoder.finish().expect("Error: Cannot finish zstd encoding on chunk data");
                },
                Err(e) => {
                    panic!("Could not create file to write chunk, {:?}", e)
                }
            }
            self.stats.add_new_item(bytes.len() as u64);
        };

        let mut hash_bytes: [u8;32] = [0;32];
        hasher.result(&mut hash_bytes);
        hash_bytes
    }
}

pub fn create_chunk_file(filename: PathBuf) -> Result<File, Error> {
    File::create(filename)
}