use std::sync::OnceLock;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

use esf_data::{Error, InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{Fit, Options};
use esf_format::esi::EsiFitting;
use esf_format::killmail::EsiKillmail;

/// The SDE is handed over once and then read straight out of Rust memory, so
/// no lookup crosses back into Python.
static SDE_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static SDE: OnceLock<Sde<'static>> = OnceLock::new();

/// `names.dat` is only read for a name `sde.dat` does not know, so it stays
/// optional.
static NAMES_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static NAMES: OnceLock<Names<'static>> = OnceLock::new();

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

/// Load `names.dat`, so an EFT written in another language than English also
/// imports. Optional; without it only English names match.
#[pyfunction]
fn load_names(bytes: Vec<u8>) -> PyResult<i32> {
    let sde = sde()?;

    if NAMES.get().is_some() {
        return Err(PyRuntimeError::new_err("names are already loaded"));
    }

    let bytes = NAMES_BYTES.get_or_init(|| bytes);
    let names = Names::new(bytes).map_err(|error| PyValueError::new_err(error.to_string()))?;

    /* Reject a mismatched pair here, rather than on every load_eft(). */
    let build_number = names.build_number();
    if sde.build_number() != build_number {
        let error = Error::BuildMismatch {
            sde: sde.build_number(),
            names: build_number,
        };
        return Err(PyValueError::new_err(error.to_string()));
    }

    let _ = NAMES.set(names);

    Ok(build_number)
}

/// Load a fit from EFT, the text format EVE copies a fit to the clipboard in.
#[pyfunction]
fn load_eft(py: Python<'_>, eft: String) -> PyResult<Bound<'_, PyAny>> {
    let sde = sde()?;
    let names = NAMES.get();

    let fit = py.detach(|| {
        let info = InfoNameSde::new(sde, names)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        esf_format::eft::load_eft(&info, &eft)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    })?;

    Ok(pythonize(py, &fit)?)
}

/// Write a fit as EFT, the text format EVE copies a fit to the clipboard in.
#[pyfunction]
fn save_eft(py: Python<'_>, fit: &Bound<'_, PyAny>) -> PyResult<String> {
    let sde = sde()?;

    let fit: Fit = depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;

    py.detach(|| {
        let info = InfoSde::new(sde);
        esf_format::eft::save_eft(&info, &fit)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    })
}

/// Load a fit from an ESI fitting, the fits a character saves in game.
#[pyfunction]
fn load_esi_fitting<'py>(
    py: Python<'py>,
    fitting: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;

    let fitting: EsiFitting =
        depythonize(fitting).map_err(|error| PyValueError::new_err(error.to_string()))?;

    let fit = py.detach(|| {
        let info = InfoSde::new(sde);
        esf_format::esi::load_esi_fitting(&info, &fitting)
    });

    Ok(pythonize(py, &fit)?)
}

/// Write a fit as an ESI fitting, the fits a character saves in game.
#[pyfunction]
fn save_esi_fitting<'py>(py: Python<'py>, fit: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;

    let fit: Fit = depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;

    let fitting = py.detach(|| {
        let info = InfoSde::new(sde);
        esf_format::esi::save_esi_fitting(&info, &fit)
    });

    Ok(pythonize(py, &fitting)?)
}

/// Load the fit of the ship that died from a killmail, as ESI returns it.
#[pyfunction]
fn load_killmail<'py>(
    py: Python<'py>,
    killmail: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;

    let killmail: EsiKillmail =
        depythonize(killmail).map_err(|error| PyValueError::new_err(error.to_string()))?;

    let fit = py.detach(|| {
        let info = InfoSde::new(sde);
        esf_format::killmail::load_killmail(&info, &killmail)
    });

    Ok(pythonize(py, &fit)?)
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
    module.add_function(wrap_pyfunction!(load_names, module)?)?;
    module.add_function(wrap_pyfunction!(load_eft, module)?)?;
    module.add_function(wrap_pyfunction!(save_eft, module)?)?;
    module.add_function(wrap_pyfunction!(load_esi_fitting, module)?)?;
    module.add_function(wrap_pyfunction!(save_esi_fitting, module)?)?;
    module.add_function(wrap_pyfunction!(load_killmail, module)?)?;
    module.add_function(wrap_pyfunction!(calculate, module)?)?;
    module.add_function(wrap_pyfunction!(beacon, module)?)?;
    Ok(())
}
