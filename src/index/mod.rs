use std::fs::File;

pub struct LocalIndexFile {
    pub path: String
    // chunk_data: Vec<ChunkData>
}

pub trait Index {
    // Load index file for extract

    // Create index file for make
    fn create(&self, path: String);
    // Add new index entry
    fn add_entry(&mut self);
}

impl Index for LocalIndexFile {
    fn create(&self, path: String) {
        // Create index file
    }
    fn add_entry(&mut self) {

    }
}