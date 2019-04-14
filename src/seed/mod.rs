use std::fs::File;
use std::io::SeekFrom;
use std::io;
use std::io::prelude::*;
use std::rc::Rc;

// Seed related data
pub struct LocalSeedFile {
    pub file: Rc<File>,
    pub path: String
}
impl LocalSeedFile {
    pub fn new(path: &str) -> LocalSeedFile {
        // read file
        match File::open(path) {
            Ok(f) => {
                LocalSeedFile {
                    path: String::from(path),
                    file: Rc::new(f)
                }
            },
            Err(e) => {
                panic!("Error: Unable to read local seed file: {:?}", e)
            }
        }
    }

    pub fn read_chunk(&mut self, start: u64, size: u64) -> Vec<u8>{
        let mut buf = Vec::new();
        let mut file = Rc::get_mut(&mut self.file).unwrap();
        file.seek(SeekFrom::Start(start)).unwrap();
        io::copy(&mut std::io::Read::by_ref(&mut file).take(size), &mut buf);
        buf
    }
}