mod io;
mod proto;

use image::io::Reader as ImageReader;
use image::DynamicImage;
use nshare::ToNdarray3;
use numpy::IntoPyArray;
use pyo3::prelude::*;
use std::io::Cursor;
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

    fn parse_scalar(&self, x: f32, py: Python) -> PyObject {
        x.into_py(py)
    }

    fn parse_image(&self, data: proto::summary::Image, py: Python) -> PyObject {
        println!("Got {:}", data.colorspace);
        // TODO Proper error handling
        // TODO Remove copies
        let img = ImageReader::new(Cursor::new(&data.encoded_image_string))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        let array = match img {
            DynamicImage::ImageRgb8(img) => img.into_ndarray3(),
            DynamicImage::ImageRgba8(img) => img.into_ndarray3(),
            _ => panic!("Unsupported image type"),
        };

        array.into_pyarray(py).to_object(py)
    }
}

#[pymethods]
impl SummaryIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        let value = slf.fetch_next_valid_value()?;

        match value {
            io::Value::SimpleValue(x) => Some(slf.parse_scalar(x, slf.py())),
            io::Value::Image(img) => Some(slf.parse_image(img, slf.py())),
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
