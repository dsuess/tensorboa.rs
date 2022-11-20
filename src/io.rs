use crate::proto;
use crc32c::crc32c;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use nshare::ToNdarray3;
use numpy::ndarray::Array3;
use prost::Message;
use std::fs::{File, OpenOptions};
use std::io::Cursor;
use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::iter::Iterator;
use std::num::Wrapping;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct RecordReader {
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

#[derive(PartialEq, Debug)]
pub enum Value {
    Scalar(f32),
    Image(Array3<u8>),
}

#[derive(PartialEq, Debug)]
pub struct Entry {
    // TODO Can we infer this from the proto?
    pub tag: String,
    pub step: i64,
    pub wall_time: f64,
    pub value: Value,
}

fn parse_image(img: proto::summary::Image) -> Array3<u8> {
    // TODO Proper error handling
    // TODO Remove copies
    let img = ImageReader::new(Cursor::new(&img.encoded_image_string))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    match img {
        // FIXME Return image in HWC channel order!
        DynamicImage::ImageRgb8(img) => img.into_ndarray3(),
        DynamicImage::ImageRgba8(img) => img.into_ndarray3(),
        _ => panic!("Unsupported image type"),
    }
}

pub fn parse_summary(elem: &[u8]) -> Option<Entry> {
    // TODO Better error handling, this parser might not even be
    let event = match proto::Event::decode(elem) {
        Ok(event) => event,
        Err(_) => return None,
    };
    let what = event.what?;

    let mut vals = match what {
        proto::event::What::Summary(summary) => summary,
        // TODO Shall we handle other entries as well
        _ => return None,
    }
    .value;

    let value = match vals.len() {
        0 => None,
        1 => Some(vals.remove(0)),
        _ => panic!("Cant deal with more than one value"), // FIXME
    }?;

    let parsed_value: Value = match value.value? {
        proto::summary::value::Value::SimpleValue(x) => Some(Value::Scalar(x)),
        proto::summary::value::Value::Image(img) => Some(Value::Image(parse_image(img))),
        _ => None,
    }?;

    let result = Entry {
        step: event.step,
        wall_time: event.wall_time,
        tag: value.tag,
        value: parsed_value,
    };
    Some(result)
}

#[cfg(test)]
mod reader_tests {
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

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_scalar_parser() {
        let path = Path::new("data/events.out.tfevents.1661684667.applepie4");
        let reader = RecordReader::new(path).unwrap();

        let mut iter = reader.into_iter();
        let elem = iter.next().unwrap().unwrap();
        assert_eq!(parse_summary(&elem), None); // FileInfo

        for (idx, buf) in iter.enumerate() {
            let entry = parse_summary(&buf.unwrap()).unwrap();
            let val = match entry.value {
                Value::Scalar(val) => val,
                _ => panic!("Invalid data found in log file"),
            };
            assert_eq!(val, idx as f32);
        }
    }
}
