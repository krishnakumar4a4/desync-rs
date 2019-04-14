use std::fs::{DirBuilder, File};
use crypto::sha2::Sha512Trunc256;
use crypto::digest::Digest;
use std::path::{Path, PathBuf};
use zstd::Encoder;
use std::io;
use std::io::Error;
use log::{info, debug};
use url::{Url, ParseError};
use crate::utils;
use std::io::Read;
use zstd::Decoder;

pub fn get_suitable_store(path: &str, min: u64, max: u64, avg: u64) -> Box<Store> {
    match Url::parse(String::from(path).trim_end_matches("/")) {
        Ok(url) => {
            if url.scheme() == "http" || url.scheme() == "https" {
                info!("path {}", path);
                Box::new(RemoteHTTPStore::new(path,min, max, avg))
            } else {
                panic!("store scheme not supported")
            }
        },
        Err(_) => {
            info!("localfile system store");
            Box::new(LocalStore::new(path, min, max, avg))
        }
    }
}

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


pub trait Store {
    fn create(&self, path: &str) -> PathBuf;
    fn write_item(&mut self, bytes: Vec<u8>) -> [u8;32];
    fn read_item(&mut self, id: Vec<u8>) -> Vec<u8>;
}

// LocalStore 
pub struct LocalStore {
    pub path: String,
    pub stats: StoreStats
}

impl LocalStore {
    pub fn new(path: &str, min:u64, max: u64, avg: u64) -> LocalStore {
        LocalStore {
            path: String::from(path),
            stats: StoreStats::new(min, max, avg)
        }
    }
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

    fn read_item(&mut self, id: Vec<u8>) -> Vec<u8> {
        use rustc_serialize::hex::ToHex;
        let chunk_name = id[..].to_hex();
        let (sub_dir_name,_) = chunk_name.split_at(4);
        let mut full_path = PathBuf::new();
        full_path.push(&self.path);
        full_path.push(sub_dir_name);
        full_path.push(chunk_name);
        full_path.set_extension("cacnk");
        info!("fullpath, {:?}",full_path);
        match File::open(full_path) {
            Ok(mut file) => {
                let mut uncompressed = Vec::new();
                let mut decoder = Decoder::new(file).unwrap();
                io::copy(&mut decoder, &mut uncompressed).expect("Error: Cannot decompress data to file");
                uncompressed
            },
            Err(e) => {
                panic!("Could not open file to read, {:?}", e);
            }
        }
    }
}

pub fn create_chunk_file(filename: PathBuf) -> Result<File, Error> {
    File::create(filename)
}

use hyper::{Client,Uri,Body,Request};
use hyper::client::{HttpConnector};
use hyper::rt::{self, Future, Stream};
use std::io::Write;
use bytes::Bytes;
use bytes::Buf;
use std::io::{Cursor};

// RemoteHTTPStore
pub struct RemoteHTTPStore {
    pub path: String,
    pub stats: StoreStats,
    pub client: Client<HttpConnector>
}

impl RemoteHTTPStore {
    pub fn new(path: &str, min: u64, max: u64, avg: u64) -> RemoteHTTPStore {
        let client = Client::new();
        RemoteHTTPStore {
            path: String::from(path),
            stats: StoreStats::new(min, max, avg),
            client: client
        }
    }
}

impl Store for RemoteHTTPStore {
    fn create(&self, path: &str) -> PathBuf {
        let req = Request::builder()
            .method("HEAD")
            .uri(&self.path)
            .body(Body::empty()).unwrap();

        let future = self.client.request(req);
        rt::run(
            future.map(|res| {
                if ! res.status().is_success() {
                    panic!("Error while finding remote http store {:?}",res.status());    
                }
            })
            .map_err(|err| {
                panic!("Remote http store doesn't exist {:?}",err);
            })
        );
        Path::new(path).to_path_buf()
    }

    fn write_item(&mut self, bytes: Vec<u8>) -> [u8;32] {
        // TODO: Write to remote location, feature may not be needed
        [0;32]
    }

    fn read_item(&mut self, id: Vec<u8>) -> Vec<u8> {
        //TODO: Read from http store
        use rustc_serialize::hex::ToHex;
        let chunk_name = id[..].to_hex();
        let (sub_dir_name,_) = chunk_name.split_at(4);
        let mut url = Url::parse(&self.path).unwrap();
        let url_current_path = url.path();
        let mut full_path = format!("{}/{}/{}.cacnk",url_current_path, sub_dir_name, chunk_name);
        url.set_path(&full_path);
        let uri = url.as_str().parse::<Uri>().unwrap();
        let mut final_bytes: Vec<u8> = Vec::new();
        let fut = fetch_chunk(uri)
            // use the parsed vector
            .map(|data| {
                let mut bufReader = Cursor::new(data).reader();
                let mut decoder = Decoder::new(bufReader).unwrap();
                io::copy(&mut decoder, &mut final_bytes).expect("Error: Cannot decompress data");
            })
            // if there was an error print it
            .map_err(|e| {
                panic!("Remote Fetch err {:?}",e);
            });

        //TODO: Fix this lifetime issue to return bytes
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let s = runtime.block_on(fut);
        // fut.wait();
        final_bytes
    }
}

fn fetch_chunk(url: hyper::Uri) -> impl Future<Item=Bytes, Error=hyper::Error> {
    let client = Client::new();
    client
        // Fetch the url...
        .get(url)
        // And then, if we get a response back...
        .and_then(|res| {
            // asynchronously concatenate chunks of the body
            res.into_body().concat2()
        })
        // use the body after concatenation
        .and_then(|body| {
            // try to parse as json with serde_json
            Ok(body.into_bytes())
        })
}

