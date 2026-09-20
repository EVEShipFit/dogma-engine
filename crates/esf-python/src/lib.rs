use std::sync::OnceLock;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

use esf_data::{InfoSde, Sde};
use esf_dogma_engine::{Fit, Options};

/// The SDE is handed over once and then read straight out of Rust memory, so
/// no lookup crosses back into Python.
static SDE_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static SDE: OnceLock<Sde<'static>> = OnceLock::new();

fn sde() -> PyResult<&'static Sde<'static>> {
    SDE.get()
        .ok_or_else(|| PyRuntimeError::new_err("SDE is not loaded; call load_sde() first"))
}

/// Load `sde.dat`. Has to be called before `calculate`; calling it twice is an
/// error, as the first buffer is borrowed for the rest of the session.
#[pyfunction]
fn load_sde(bytes: Vec<u8>) -> PyResult<i32> {
    if SDE.get().is_some() {
        return Err(PyRuntimeError::new_err("SDE is already loaded"));
    }

    let bytes = SDE_BYTES.get_or_init(|| bytes);
    let sde = Sde::new(bytes).map_err(|error| PyValueError::new_err(error.to_string()))?;

    let build_number = sde.build_number();
    let _ = SDE.set(sde);

    Ok(build_number)
}

/// Calculate every attribute of the ship, its items and the character.
#[pyfunction]
#[pyo3(signature = (fit, options = None))]
fn calculate<'py>(
    py: Python<'py>,
    fit: &Bound<'py, PyAny>,
    options: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;

    let fit: Fit = depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;
    let options: Options = match options {
        None => Options::default(),
        Some(options) if options.is_none() => Options::default(),
        Some(options) => {
            depythonize(options).map_err(|error| PyValueError::new_err(error.to_string()))?
        }
    };

    let calculation = py.detach(|| {
        let info = InfoSde::new(sde);
        esf_dogma_engine::calculate(&info, &fit, &options)
    });

    Ok(pythonize(py, &calculation)?)
}

/// What a beacon in space hands to every fit in there with it. Put the result
/// in `incoming` of a fit to have it applied.
#[pyfunction]
fn beacon(py: Python<'_>, type_id: i32) -> PyResult<Bound<'_, PyAny>> {
    let sde = sde()?;

    let projection = py.detach(|| {
        let info = InfoSde::new(sde);
        esf_dogma_engine::beacon(&info, type_id)
    });

    Ok(pythonize(py, &projection)?)
}

#[pymodule]
fn _esf_dogma_engine(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(load_sde, module)?)?;
    module.add_function(wrap_pyfunction!(calculate, module)?)?;
    module.add_function(wrap_pyfunction!(beacon, module)?)?;
    Ok(())
}
