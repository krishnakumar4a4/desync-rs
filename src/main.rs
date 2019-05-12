// Algorithm:
    // Chunk and create new index file:
        // Read file to chunk
        // read min, max and avg values and define descriminator
        // Define buzhash table
            // Read at once "min + Window size of data" -> calculate rolling hash of it
            // Start calculating rolling hash until
            // ---> EOF
            // ---> read buffer size crossed max chunk size
            // ---> hash value equal to discriminator value
            // Calculate hash and write to file with Compression.
    // List chunk hashs, offsets and sizes
    // Chunk with existing index file
// Features to support 
// casync make with 
// --> remote and local index and chunk stores
// --> filesystems and block devices
mod index;
mod store;
mod io;
mod chunker;
mod utils;
mod assembler;
mod seed;

extern crate log;
extern crate log4rs;
extern crate clap;
extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate bytes;
extern crate tokio;

use crate::assembler::AssembleOps;
use crate::index::Index;
use log::{info};
use clap::ArgMatches;

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let mut matches = ArgMatches::new();
    utils::get_matches_from_cli(&mut matches);
    
    match matches.subcommand() {
        ("make", Some(sub_com)) => {
            let index_file_name = sub_com.value_of("index").unwrap_or("index.caibx");
            let store_folder_name = sub_com.value_of("store").unwrap_or("default.castr");
            let input_file_name = sub_com.value_of("file").unwrap();

            // TODO: Should have been Chunker instead of ChunkerConfig, separate out configuration
            let mut chunkerConfig = chunker::ChunkerConfig {
                index: Box::new(index::LocalIndexFile::new(index_file_name)),
                store: Box::new(store::LocalStore::new(store_folder_name, chunker::CHUNK_SIZE_MIN_DEFAULT, chunker::CHUNK_SIZE_MAX_DEFAULT, chunker::CHUNK_SIZE_AVG_DEFAULT)),
                source: Box::new(io::LocalSourceFile::new(String::from(input_file_name))),
                min_size: chunker::CHUNK_SIZE_MIN_DEFAULT,
                max_size: chunker::CHUNK_SIZE_MAX_DEFAULT,
                avg_size: chunker::CHUNK_SIZE_AVG_DEFAULT
            };
            chunkerConfig.chunk();
        },
        ("extract", Some(sub_com)) => {
            let index_file_name = sub_com.value_of("index").unwrap_or("index.caibx");
            let store_folder_name = sub_com.value_of("store").unwrap_or("default.castr");
            let output_file_name = sub_com.value_of("file").unwrap();
            let seed_index_file = sub_com.value_of("seed-index");
            let seed_file = sub_com.value_of("seed-file");

            let mut a = if let Some(seed_file_name) = seed_file {
                if let Some(seed_index_file_name) = seed_index_file {
                        assembler::AssemblerConfig {
                            seed: Some(seed::LocalSeedFile::new(seed_file_name)),
                            seed_index: Some(Box::new(index::LocalIndexFile::open(seed_index_file_name))),
                            store: store::get_suitable_store(store_folder_name, chunker::CHUNK_SIZE_MIN_DEFAULT, chunker::CHUNK_SIZE_MAX_DEFAULT, chunker::CHUNK_SIZE_AVG_DEFAULT),
                            new_index: Box::new(index::LocalIndexFile::open(index_file_name)),
                            output: Box::new(io::LocalOutputFile::new(output_file_name))
                        } 
                    } else {
                        info!("Ignoring seed, seed_index");        
                        assembler::AssemblerConfig {
                            seed: None,
                            seed_index: None,
                            store: store::get_suitable_store(store_folder_name, chunker::CHUNK_SIZE_MIN_DEFAULT, chunker::CHUNK_SIZE_MAX_DEFAULT, chunker::CHUNK_SIZE_AVG_DEFAULT),
                            new_index: Box::new(index::LocalIndexFile::open(index_file_name)),
                            output: Box::new(io::LocalOutputFile::new(output_file_name))
                        }
                    }
            } else {
                info!("Ignoring seed and continuing, both seed and index are required");
                assembler::AssemblerConfig {
                    seed: None,
                    seed_index: None,
                    store: store::get_suitable_store(store_folder_name, chunker::CHUNK_SIZE_MIN_DEFAULT, chunker::CHUNK_SIZE_MAX_DEFAULT, chunker::CHUNK_SIZE_AVG_DEFAULT),
                    new_index: Box::new(index::LocalIndexFile::open(index_file_name)),
                    output: Box::new(io::LocalOutputFile::new(output_file_name))
                }
            };
            a.assemble();
        },
        ("list-chunks", Some(sub_com)) => {
            let index_file = sub_com.value_of("index");
            let input_file = sub_com.value_of("file");
            if let Some(index_file_name) = index_file {
                let mut index_holder = index::LocalIndexFile::open(index_file_name);
                index_holder.read();
                println!("\nTotal number of chunks {}\n", index_holder.getChunkData().len());
                println!("chunk_id/start/size(bytes):\n");
                index_holder.getChunkData().iter().for_each(|chunk| {
                    println!("{:70} {:20} {:20}", utils::bytes_to_hex(chunk.id.to_vec()), chunk.start, chunk.size);
                });
                println!("Done!");
            } else if let Some(input_file_name) = input_file {
                let mut chunkerConfig = chunker::ChunkerConfig {
                    index: Box::new(index::InMemoryIndex::new("")),
                    store: Box::new(store::DummyStore::new(chunker::CHUNK_SIZE_MIN_DEFAULT, chunker::CHUNK_SIZE_MAX_DEFAULT, chunker::CHUNK_SIZE_AVG_DEFAULT)),
                    source: Box::new(io::LocalSourceFile::new(String::from(input_file_name))),
                    min_size: chunker::CHUNK_SIZE_MIN_DEFAULT,
                    max_size: chunker::CHUNK_SIZE_MAX_DEFAULT,
                    avg_size: chunker::CHUNK_SIZE_AVG_DEFAULT
                };
                chunkerConfig.chunk();
            } else {
                panic!("invalid options");
            }
        },
        _ => {
            panic!("Arg not supported/provided");
        }
    };
}
