mod io;
mod proto;

use numpy::IntoPyArray;
use pyo3::prelude::*;
use std::path::PathBuf;

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

#[pymethods]
impl SummaryIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        let value = slf.fetch_next_valid_value()?;
        let py = slf.py();

        match value {
            io::Value::Scalar(x) => Some(x.into_py(py)),
            io::Value::Image(img) => Some(img.into_pyarray(py).to_object(py)),
        }
    }
}

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
