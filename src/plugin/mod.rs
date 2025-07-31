use std::path::Path;
use std::ffi::CString;

use pyo3::{exceptions::PyValueError, PyErr};
use pyo3::exceptions::{PyFileNotFoundError, PyIOError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

use crate::plugin::error::IntegrityError;

pub mod error;
pub mod interface;


/// 
/// Loads a plugin, given a path, as a Pyo3 Python module.
/// The function includes path integrity checks.
///
/// # Parameters:
/// - `py`: The Python global interpreter lock
/// - `path`: The path of the plugin file
///
/// # Returns:
/// - A PyModule if the file was loaded successfully
/// - A PyErr if there was an issue with opening the module
///
pub fn load_plugin_module<'py>(py: Python<'py>, path: &str) -> Result<Bound<'py, PyModule>, PyErr> {
    let src_path = Path::new(path);

    match src_path.try_exists() {
        Ok(exists) => {
            if !exists {
                return Err(PyFileNotFoundError::new_err(format!(
                    "File not found: {}",
                    path
                )));
            }
        },
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Error checking files existance: {}",
                err.to_string()
            )));
        }
    }


    let code = match std::fs::read_to_string(path) {
        Ok(str) => str,
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Failed to read Python file {}: {}",
                path, err.to_string()
            )));
        }
    };

    let module_name = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or("plugin");

    let c_code = match make_cstr(&code) {
        Ok(val) => val,
        Err(err) => { return Err(PyErr::new::<PyValueError, _>(format!("Error making code cstr: {}", err.to_string()))); }
    };

    let c_path = match make_cstr(path) {
        Ok(val) => val,
        Err(err) => { return Err(PyErr::new::<PyValueError, _>(format!("Error making path cstr: {}", err.to_string()))); }
    };

    let c_module_name = match make_cstr(module_name) {
        Ok(val) => val,
        Err(err) => { return Err(PyErr::new::<PyValueError, _>(format!("Error making module cstr: {}", err.to_string()))); }
    };


    match PyModule::from_code(py, &c_code, &c_path, &c_module_name) {
        Ok(val) => { Ok(val) },
        Err(err) => Err(err)
    }
}


/// 
/// Verifies the integrity of a plugin.
/// At the moment, all plugins require a params and run method.
/// 
/// Parameters:
/// - `module`: The Python module
///
/// Returns:
/// - Void if the plugin appears correct
/// - An error if the plugin integrity check failed
///
pub fn verify_plugin<'py>(module: &Bound<'py, PyModule>) -> Result<(), IntegrityError> {

    const REQUIRED_FUNCS: [&str; 2] = ["params", "run"];

    for name in REQUIRED_FUNCS {
        match module.hasattr(name) {
            Ok(bool) => {
                if !bool {
                    return Err(IntegrityError::MissingFunction { func_name: name.to_string() });
                }
            },
            Err(err) => {
                return Err(IntegrityError::PyErr { err: err });
            }
        }

    }

    Ok(())
}


/// 
/// Grabs the return value of the `params` function of a plugin.
/// This value is used in the frontend to display the parameters of a plugin for editing by the end user.
///
/// # Parameters:
/// - `path`: The path to the Python plugin
///
/// # Returns:
/// - A string, the return of the `params` method
/// - A string explaining why the function failed
///
pub fn get_parameter_string<'py>(path: &str) -> Result<String, String> {
     match Python::with_gil(|py| {
        let module = match load_plugin_module(py, path) {
            Ok(val) => val,
            Err(err) => { return Err(err.to_string()); }
        };

        let result = verify_plugin(&module);
        match result {
            Ok(()) => {},
            Err(err) => { return Err(err.to_string()); }
        };

        // since the module is okay, we'll grab the parameter string
        
        let parameter_fn = module.getattr("params").unwrap();
        let result = match parameter_fn.call0() {
            Ok(result) => result,
            Err(err) => {
                return Err(format!("Error running `run` function in plugin: {}", err.to_string()));
            }
        };

        Ok(result.to_string())
    }) {
        Ok(ins) => Ok(ins),
        Err(err) => Err(format!("Error getting plugin parameters: {}", err.to_string()))
    }
}


/// 
/// Loads a string into a PyDict, using the Python global interpreter to 
/// execute json.loads(str) on the input string.
///
/// # Parameters:
/// - `py`: The Python GIL
/// - `json_str`: The string, in JSON format, to be turned into a Python dict
///
/// # Returns:
/// - A Py<PyDict> if successful
/// - A PyErr explaning why the function failed
///
pub fn json_loads<'py>(py: Python<'py>, json_str: &str) -> PyResult<Py<PyDict>> {
    let json_module = PyModule::import(py, "json").unwrap();
    let loads_fn = json_module.getattr("loads").unwrap();

    let py_dict: Py<PyDict> = loads_fn.call1((json_str,))?.extract()?;

    Ok(py_dict)
}


/// 
/// Generates a CString from a given &str.
///
/// # Parameters:
/// - `str`: The string to turn into a CString
///
/// # Returns:
/// - A CString
/// - A PyErr explaining why the function failed
///
pub fn make_cstr(str: &str) -> Result<CString, PyErr> {
    match CString::new(str) {
        Ok(val) => Ok(val),
        Err(err) => {
            Err(
                PyValueError::new_err(
                    format!("Error making CStr from {}: {}", str, err.to_string())
                )
            )    
        }
    }
}
