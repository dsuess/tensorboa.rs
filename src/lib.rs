mod io;
mod proto;

use pyo3::{exceptions::PyRuntimeError, prelude::*, PyIterProtocol};
use std::{iter::Sum, path::PathBuf};

#[pyclass(unsendable)]
struct SummaryIterator {
    reader: io::RecordReader,
    parser: io::SummaryParser,
}

impl SummaryIterator {
    fn fetch_next_valid_value(&mut self) -> Option<io::Value> {
        loop {
            // TODO Better error handling
            // Return if reader is finished
            let data = self.reader.next()?.unwrap();
            // Skip all elems that are converted to None by the parser
            match self.parser.parse(&data) {
                Some(val) => return Some(val.value),
                None => continue,
            };
        }
    }
}

#[pyproto]
impl PyIterProtocol for SummaryIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<f32> {
        let value = slf.fetch_next_valid_value()?;

        match value {
            io::Value::SimpleValue(x) => Some(x),
            _ => panic!("Cant convert {:?} to pyvalue", value),
        }
    }
}

/// Formats the sum of two numbers as string.
#[pyclass(unsendable)]
struct SummaryReader {
    path: PathBuf,
}

#[pymethods]
impl SummaryReader {
    #[new]
    // FIXME Can we accept both str and pathlib.Path
    fn new(path: String) -> PyResult<Self> {
        return Ok(Self {
            path: PathBuf::from(&path),
        });
    }
}

#[pyproto]
impl PyIterProtocol for SummaryReader {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<SummaryIterator>> {
        let result = SummaryIterator {
            reader: io::RecordReader::new(&slf.path)?.into_iter(),
            parser: io::SummaryParser {},
        };
        Py::new(slf.py(), result)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tensorboars(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SummaryReader>()?;
    Ok(())
}
