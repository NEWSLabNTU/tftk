mod serde;
mod types;

use pyo3::prelude::*;

pub use crate::{
    serde::{dump_tf, dumps_tf, load_tf, loads_tf},
    types::{PyMaybeTransform, PyRotation, PyRotationForm},
};

#[pymodule]
fn tfpy(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // classes
    m.add_class::<PyMaybeTransform>()?;
    m.add_class::<PyRotation>()?;
    m.add_class::<PyRotationForm>()?;

    // functions
    m.add_function(wrap_pyfunction!(load_tf, m)?)?;
    m.add_function(wrap_pyfunction!(loads_tf, m)?)?;
    m.add_function(wrap_pyfunction!(dump_tf, m)?)?;
    m.add_function(wrap_pyfunction!(dumps_tf, m)?)?;

    Ok(())
}
