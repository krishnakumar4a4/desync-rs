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

fn main() {
    let filename = String::from("test_files/temp/input");
    let mut chunkerConfig = chunker::ChunkerConfig {
        index: Box::new(index::LocalIndexFile {
            path: String::from("some_file")
        }),
        store: Box::new(store::LocalStore {
            path: String::from("some_store"),
            stats: store::StoreStats::new(chunker::CHUNK_SIZE_MIN_DEFAULT, 
                    chunker::CHUNK_SIZE_MAX_DEFAULT, 
                    chunker::CHUNK_SIZE_AVG_DEFAULT)
        }),
        source: Box::new(io::LocalSourceFile::new(filename))
    };

    chunkerConfig.chunk();
}
