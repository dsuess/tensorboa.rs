use crate::proto;
use prost::Message;
use std::path::Path;

pub struct SummaryParser {}

type Value = proto::summary::value::Value;

#[derive(PartialEq, Debug)]
pub struct Entry {
    // TODO Can we infer this from the proto?
    tag: String,
    step: i64,
    wall_time: f64,
    value: Value,
}

impl SummaryParser {
    pub fn parse(&self, elem: &[u8]) -> Option<Entry> {
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

        let result = Entry {
            step: event.step,
            wall_time: event.wall_time,
            tag: value.tag,
            value: value.value?,
        };
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::record_reader::RecordReader;

    use super::*;

    #[test]
    fn test_scalar_parser() {
        let path = Path::new("data/events.out.tfevents.1661684667.applepie4");
        let reader = RecordReader::new(path).unwrap();
        let parser = SummaryParser {};

        let mut iter = reader.into_iter();
        let elem = iter.next().unwrap().unwrap();
        assert_eq!(parser.parse(&elem), None); // FileInfo

        for (idx, buf) in iter.enumerate() {
            let entry = parser.parse(&buf.unwrap()).unwrap();
            let val = match entry.value {
                Value::SimpleValue(val) => val,
                _ => panic!("Invalid data found in log file"),
            };
            assert_eq!(val, idx as f32);
        }
    }
}
