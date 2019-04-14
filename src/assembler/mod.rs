use crate::io as local_io;
use crate::index;
use crate::seed;
use crate::store;

use log::{debug, info};
use std::rc::Rc;

// AssemblerConfig 
pub struct AssemblerConfig {
    pub seed: Option<seed::LocalSeedFile>,
    pub seed_index: Option<Box<index::Index>>,
    pub store: Box<store::Store>,
    pub new_index: Box<index::Index>,
    pub output: Box<local_io::LocalOutputFile>
    // Add store here
}

pub trait AssembleOps {
    fn assemble(&mut self);
}

impl AssembleOps for AssemblerConfig {
    fn assemble(&mut self) {
        // Read the index file 
        self.new_index.read();

        let chunks_updated = self.new_index.getChunkData();
        let out_file = Rc::get_mut(&mut self.output.file).unwrap();
        
        info!("Started assembling");
        match &mut self.seed_index {
            Some(seed_index) => {
                info!("Found seed index");
                seed_index.read();
                let chunks_from_seed = seed_index.getChunkData();
                match &mut self.seed {
                    Some(seed) => {
                        for uc in chunks_updated.iter() {
                            let mut should_get_chunk = true;
                            for c in chunks_from_seed.iter() {
                                if c.id == uc.id {
                                    //Read the chunk and assemble
                                    info!("Getting chunk from seed");
                                    let buf = seed.read_chunk(c.start, c.size);
                                    self.output.write_all(buf);
                                    should_get_chunk = false;
                                    break;
                                }
                            }
                            if should_get_chunk {
                                // Get chunks from update
                                info!("Could not find chunk in the seed, Getting chunk from store");
                                let chunk_bytes = self.store.read_item(uc.id.to_vec());
                                self.output.write_all(chunk_bytes);
                            }
                        }
                    },
                    None => {
                        panic!("Seed file needed");
                    }
                }
            },
            None => {
                for uc in chunks_updated.iter() {
                    // Should download chunks
                    info!("Getting chunk from store");
                    let chunk_bytes = self.store.read_item(uc.id.to_vec());
                    self.output.write_all(chunk_bytes);
                }
            }
        }
    }
}