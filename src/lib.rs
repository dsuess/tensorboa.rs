mod io;
mod proto;

use numpy::IntoPyArray;
use pariter::IteratorExt;
use pyo3::prelude::*;

use std::io::Result;
use std::path::PathBuf;

#[pyclass(unsendable)]
struct SummaryIterator {
    iterator: Box<dyn Iterator<Item = Option<io::Entry>>>,
}

#[pymethods]
impl SummaryIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        // Pull items from iterator until next valid element is reached
        let value = loop {
            // Single question mark to signify end of loop
            match slf.iterator.next()? {
                Some(val) => break val,
                _ => continue,
            }
        };

        match value.value {
            io::Value::Scalar(x) => Some(x.into_py(slf.py())),
            io::Value::Image(img) => Some(img.into_pyarray(slf.py()).to_object(slf.py())),
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
        fn parse(elem: Result<Vec<u8>>) -> Option<io::Entry> {
            io::parse_summary(&elem.unwrap())
        }

        let iterator = io::RecordReader::new(&slf.path)?
            .into_iter()
            .filter(|elem: &Result<Vec<u8>>| -> bool { elem.is_ok() })
            .parallel_map(parse);

        let result = SummaryIterator {
            iterator: Box::new(iterator),
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
