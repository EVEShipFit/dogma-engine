use std::sync::OnceLock;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

use esf_data::{Error, InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{Fit, Options};
use esf_format::esf;
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

/// Set a fit loaded from EFT or an ESI fitting to the states EVE gives it on
/// import. Set the skills of the character first.
#[pyfunction]
fn post_load<'py>(py: Python<'py>, fit: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;

    let mut fit: Fit =
        depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;

    py.detach(|| {
        let info = InfoSde::new(sde);
        esf_format::post_load(&info, &mut fit);
    });

    Ok(pythonize(py, &fit)?)
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

/// Load a fit from an EVEShip.fit link, given its version and its payload
/// once unbase64'd and gunzipped.
#[pyfunction]
fn load_link<'py>(
    py: Python<'py>,
    version: String,
    payload: String,
) -> PyResult<Bound<'py, PyAny>> {
    let sde = sde()?;
    let names = NAMES.get();

    let fit = py.detach(|| {
        let info = InfoNameSde::new(sde, names)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        esf_format::link::load_link(&info, &version, &payload)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    })?;

    Ok(pythonize(py, &fit)?)
}

fn esf_error(error: esf::Error) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn load_esf_fit(sde: &'static Sde<'static>, data: &[u8]) -> PyResult<Fit> {
    let info = InfoSde::new(sde);

    let fits = esf::load_esf(&info, data).map_err(esf_error)?;
    let fit =
        esf::main_fit(&fits).ok_or_else(|| PyValueError::new_err("no fit stands on its own"))?;
    esf::to_fit(&info, fit).map_err(esf_error)
}

/// Load a fit from an esf/1 document, as text.
#[pyfunction]
fn load_esf(py: Python<'_>, text: String) -> PyResult<Bound<'_, PyAny>> {
    let sde = sde()?;

    let fit = py.detach(|| load_esf_fit(sde, text.as_bytes()))?;

    Ok(pythonize(py, &fit)?)
}

/// Load a fit from an esf/1 link: the binary form, in base64url.
#[pyfunction]
fn load_esf_link(py: Python<'_>, link: String) -> PyResult<Bound<'_, PyAny>> {
    let sde = sde()?;

    let fit = py.detach(|| load_esf_fit(sde, &esf::decode_base64url(&link).map_err(esf_error)?))?;

    Ok(pythonize(py, &fit)?)
}

/// Write a fit as an esf/1 document, as text.
#[pyfunction]
fn save_esf(py: Python<'_>, fit: &Bound<'_, PyAny>) -> PyResult<String> {
    let sde = sde()?;

    let fit: Fit = depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;

    py.detach(|| {
        let info = InfoSde::new(sde);
        let esf_fit = esf::from_fit(&info, &fit).map_err(esf_error)?;
        esf::save_esf(&info, &[esf_fit]).map_err(esf_error)
    })
}

/// Write a fit as an esf/1 link: the binary form, in base64url.
#[pyfunction]
fn save_esf_link(py: Python<'_>, fit: &Bound<'_, PyAny>) -> PyResult<String> {
    let sde = sde()?;

    let fit: Fit = depythonize(fit).map_err(|error| PyValueError::new_err(error.to_string()))?;

    py.detach(|| {
        let info = InfoSde::new(sde);
        let esf_fit = esf::from_fit(&info, &fit).map_err(esf_error)?;
        let binary = esf::save_esf_binary(&info, &[esf_fit]).map_err(esf_error)?;
        Ok(esf::encode_base64url(&binary))
    })
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
    module.add_function(wrap_pyfunction!(post_load, module)?)?;
    module.add_function(wrap_pyfunction!(load_killmail, module)?)?;
    module.add_function(wrap_pyfunction!(load_link, module)?)?;
    module.add_function(wrap_pyfunction!(load_esf, module)?)?;
    module.add_function(wrap_pyfunction!(load_esf_link, module)?)?;
    module.add_function(wrap_pyfunction!(save_esf, module)?)?;
    module.add_function(wrap_pyfunction!(save_esf_link, module)?)?;
    module.add_function(wrap_pyfunction!(calculate, module)?)?;
    module.add_function(wrap_pyfunction!(beacon, module)?)?;
    Ok(())
}
