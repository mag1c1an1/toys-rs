use std::fmt::Display;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyclass]
struct Element {
    atomic_number: u32,
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Element {}", self.atomic_number)
    }
}

#[pymethods]
impl Element {
    #[new]
    fn new(atomic_number: u32) -> Self {
        Element { atomic_number }
    }

    #[staticmethod]
    fn o() -> Self {
        Element { atomic_number: 8 }
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn ffi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Element>()?;
    Ok(())
}
