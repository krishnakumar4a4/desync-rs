use std::io::{Write, Read};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{ErrorKind, Error};
use std::fs::File;

pub fn read_u64(f: &mut File) -> u64 {
    let mut buf = [0;8];
    f.read_exact(&mut buf).unwrap();
    LittleEndian::read_u64(&buf)
}

pub fn read_32_bytes(f: &mut File, buf: &mut [u8;32]) -> Result<(),Error> {
    f.read_exact(buf)
}

pub fn write_u64(f: &mut File, value: u64) -> Result<(),Error> {
    let mut buf = [0;8];
    LittleEndian::write_u64(&mut buf, value);
    f.write_all(&buf)
}

pub fn write_32_bytes(f: &mut File, buf: [u8;32]) -> Result<(),Error> {
    f.write_all(&buf)
}

// TODO: Figure out why this not equivalent using rustc-serialize::hex::ToHex?
pub fn bytes_to_hex(bytes: Vec<u8>) -> String{
    bytes.iter().map(|b|{
        format!("{:x?}",b)
    }).collect()
}

// -----------------------------------------------------------------------------------------------
// Argument Parsing
// -----------------------------------------------------------------------------------------------
use clap::{SubCommand, Arg, App, ArgMatches, ArgGroup};

pub fn get_matches_from_cli(arg_matches: &mut ArgMatches) {
    *arg_matches = App::new("desync-rs")
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
                        )
        .subcommand(SubCommand::with_name("list-chunks")
                        .help("List chunks from a given index file or an input file")
                        .arg(Arg::with_name("index")
                                .short("i")
                                .long("index")
                                .help("Path to index file")
                                .takes_value(true))
                        .arg(Arg::with_name("file")
                                .short("f")
                                .long("file")
                                .help("Path to input file")
                                .takes_value(true))
                        .group(ArgGroup::with_name("either_of_args")
                                .args(&["index", "file"])
                                .required(true))                     
                        )
        .subcommand(SubCommand::with_name("verify-index")
                        .help("Verifies a given index file against the input file")
                        .arg(Arg::with_name("index")
                                .short("i")
                                .long("index")
                                .help("Path to index file")
                                .takes_value(true)
                                .required(true))
                        .arg(Arg::with_name("f")
                                .short("f")
                                .long("file")
                                .help("Path to input file")
                                .required(true))
                        ).get_matches();

        // TODO: Compare indexes and their correspondig sizes
        // TODO: Prune suuport
}