mod io;
mod proto;

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::path::PathBuf;

/// Formats the sum of two numbers as string.
#[pyclass(unsendable)]
struct SummaryReader {
    path: PathBuf,
    reader: Option<io::RecordReader>,
    parser: io::SummaryParser,
}

#[pymethods]
impl SummaryReader {
    #[new]
    // FIXME Can we accept both str and pathlib.Path
    fn new(path: String) -> PyResult<Self> {
        let parser = return Ok(Self {
            path: PathBuf::from(&path),
            reader: None,
            parser: io::SummaryParser {},
        });
    }

    fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        if slf.reader.is_some() {
            return Err(PyRuntimeError::new_err("Tried to reenter"));
        }

        slf.reader = Some(io::RecordReader::new(&slf.path)?);
        Ok(slf)
    }

    fn __exit__(
        mut slf: PyRefMut<'_, Self>,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<PyRefMut<'_, Self>> {
        if slf.reader.is_none() {
            return Err(PyRuntimeError::new_err("Tried to exit outside enter"));
        }

        slf.reader = None;
        Ok(slf)
    }
}
/// A Python module implemented in Rust.
#[pymodule]
fn tensorboars(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SummaryReader>()?;
    Ok(())
}
