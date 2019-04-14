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

use clap::{SubCommand, Arg, App};
use crate::assembler::AssembleOps;
use log::{info};

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let matches = App::new("desync-rs")
        .version("0.1.0")
        .author("Krishna Kumar T <krishna.thokala2010@gmail.com>")
        .subcommand(SubCommand::with_name("make")
                    .help("Creates chunks for the input file")
                    .arg(Arg::with_name("index")
                            .short("i")
                            .long("index")
                            .help("Path to index file")
                            .takes_value(true))
                    .arg(Arg::with_name("store")
                            .short("s")
                            .long("store")
                            .help("Path to chunk store")
                            .takes_value(true))
                    .arg(Arg::with_name("file")
                            .short("f")
                            .long("file")
                            .help("Path to input file to be chunked")
                            .takes_value(true)
                            .required(true))
                        )
        .subcommand(SubCommand::with_name("extract")
                    .help("Assembles chunks to form output")
                    .arg(Arg::with_name("index")
                            .short("i")
                            .long("index")
                            .help("Path to index file")
                            .takes_value(true)
                            .required(true))
                    .arg(Arg::with_name("store")
                            .short("s")
                            .long("store")
                            .help("Path to chunk store")
                            .takes_value(true))
                    .arg(Arg::with_name("seed-file")
                            .long("sf")
                            .help("Path to seed file")
                            .takes_value(true))
                    .arg(Arg::with_name("seed-index")
                            .long("si")
                            .help("Path to seed index file")
                            .takes_value(true))
                    .arg(Arg::with_name("file")
                            .short("f")
                            .long("file")
                            .help("Path to input file to be chunked")
                            .takes_value(true)
                            .required(true))
                    ).get_matches();
    match matches.subcommand() {
        ("make", Some(sub_com)) => {
            let index_file_name = sub_com.value_of("index").unwrap_or("index.caibx");
            let store_folder_name = sub_com.value_of("store").unwrap_or("default.castr");
            let input_file_name = sub_com.value_of("file").unwrap();

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
                info!("Ignoring seed, both seed and index or required");
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
        _ => {
            panic!("Arg not supported/provided");
        }
    };
}
