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
            // Calculate hash and write to file // Compression needed?
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

extern crate log;
extern crate log4rs;
extern crate clap;

use clap::{SubCommand, Arg, App};

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
                        ).get_matches();
    let mut chunkerConfig = match matches.subcommand() {
        ("make", Some(sub_com)) => {
            let index_file_name = sub_com.value_of("index").unwrap_or("index.caibx");
            let store_folder_name = sub_com.value_of("store").unwrap_or("default.castr");
            let input_file_name = sub_com.value_of("file").unwrap();

            chunker::ChunkerConfig {
                index: Box::new(index::LocalIndexFile::new(index_file_name)),
                store: Box::new(store::LocalStore {
                    path: String::from(store_folder_name),
                    stats: store::StoreStats::new(chunker::CHUNK_SIZE_MIN_DEFAULT,
                            chunker::CHUNK_SIZE_MAX_DEFAULT,
                            chunker::CHUNK_SIZE_AVG_DEFAULT)
                }),
                source: Box::new(io::LocalSourceFile::new(String::from(input_file_name))),
                min_size: chunker::CHUNK_SIZE_MIN_DEFAULT,
                max_size: chunker::CHUNK_SIZE_MAX_DEFAULT,
                avg_size: chunker::CHUNK_SIZE_AVG_DEFAULT
            }
        },
        _ => {
            panic!("Arg not supported/provided");
        }
    };

    chunkerConfig.chunk();
}
