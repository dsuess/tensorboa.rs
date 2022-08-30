use crc32c::crc32c;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::iter::Iterator;
use std::num::Wrapping;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

struct RecordReader {
    buf: BufReader<File>,
    last_elem_valid: bool,
}

fn masked_crc32c(data: &[u8]) -> u32 {
    let checksum = Wrapping(crc32c(data));
    let offset = Wrapping(0xA282EAD8 as u32);
    (((checksum >> 15) | (checksum << 17)) + offset).0
}

impl RecordReader {
    pub fn new(path: &Path) -> Result<RecordReader> {
        let f = OpenOptions::new().read(true).open(path)?;
        let buf = BufReader::new(f);
        Ok(RecordReader {
            buf: buf,
            last_elem_valid: true,
        })
    }

    fn parse_next_elem(buf: &mut BufReader<File>) -> Result<Vec<u8>> {
        let header = buf.read_u64::<LittleEndian>()?;
        // TODO Abstract read_bytes into function/Trait
        let crc_header = buf.read_u32::<LittleEndian>()?;
        let bytes: [u8; 8] = header.to_le_bytes();
        let checksum = masked_crc32c(&bytes);
        if checksum != crc_header {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Header checksum mismatch",
            ));
        }

        let mut data: Vec<u8> = vec![0; header as usize];
        buf.read_exact(data.as_mut_slice())?;
        let crc_data = buf.read_u32::<LittleEndian>()?;
        let checksum = masked_crc32c(&data);
        if checksum != crc_data {
            return Err(Error::new(ErrorKind::InvalidData, "Data checksum mismatch"));
        }
        Ok(data)
    }
}

impl Iterator for RecordReader {
    type Item = Result<Vec<u8>>;

    //      handled correctly :(
    fn next(&mut self) -> Option<Self::Item> {
        if !self.last_elem_valid {
            return None;
        }

        match Self::parse_next_elem(&mut self.buf) {
            Ok(val) => Some(Ok(val)),
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => {
                self.last_elem_valid = false;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_masked_crc32() {
        let bytes: [u8; 8] = (24 as u64).to_le_bytes();
        assert_eq!(masked_crc32c(&bytes), 575373219);
    }

    #[test]
    fn test_correct_count() {
        // TODO Dynamically create using python
        let path = Path::new("data/events.out.tfevents.1661684667.applepie4");
        let reader = RecordReader::new(path).unwrap();
        // 10 logged entries + init event
        let maybe_elems: Result<Vec<_>> = reader.into_iter().collect();

        match maybe_elems {
            Ok(elems) => assert!(elems.len() == 11),
            Err(e) => panic!("{}", e),
        }
    }
}
