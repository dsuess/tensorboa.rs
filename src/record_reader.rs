use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Read, Result};
use std::iter::Iterator;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

struct RecordReader {
    buf: BufReader<File>,
}

impl RecordReader {
    pub fn new(path: &Path) -> Result<RecordReader> {
        let f = OpenOptions::new().read(true).open(path)?;
        let buf = BufReader::new(f);
        Ok(RecordReader { buf })
    }

    fn parse_next_elem(buf: &mut BufReader<File>) -> Result<Vec<u8>> {
        let header = buf.read_u64::<LittleEndian>()?;
        // TODO Add verification!
        let _crc_header = buf.read_u32::<LittleEndian>()?;

        // header determines number of bytes to be read
        let mut data: Vec<u8> = vec![0; header as usize];
        println!("Reading {}", header);
        buf.read_exact(data.as_mut_slice())?;
        // TODO Add verification!
        let _crc_data = buf.read_u32::<LittleEndian>()?;
        Ok(data)
    }
}

impl Iterator for RecordReader {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        match Self::parse_next_elem(&mut self.buf) {
            Ok(val) => Some(Ok(val)),
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_correct_count() {
        // TODO Dynamically create using python
        let path = Path::new("data/events.out.tfevents.1661684667.applepie4");
        let reader = RecordReader::new(path).unwrap();
        // 10 logged entries + init event
        assert_eq!(reader.into_iter().count(), 11);
    }
}
