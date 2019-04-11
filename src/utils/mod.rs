use std::io::{Write, Read};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{ErrorKind, Error};
use std::fs::File;

pub fn readU64(f: &mut File) -> u64 {
    let mut buf = [0;8];
    f.read_exact(&mut buf).unwrap();
    LittleEndian::read_u64(&buf)
}

pub fn read32Bytes(f: &mut File, buf: &mut [u8;32]) -> Result<(),Error> {
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