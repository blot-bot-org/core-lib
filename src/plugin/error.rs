use thiserror::Error;

use pyo3::PyErr;

///
/// All errors emitted from the file integrity checker.
/// The error messages can be displayed to users on the frontend. Format nicely please.
///
/// - `FileNotFound`: When the plugin file path is not found
///     Parameters:
///     - `path`: The path which was invalid
/// - `MissingFunction`: When the plugin file path does not contain all the required functions
///     Parameters:
///     - `func_name`: The function which was missing
/// - `PyErr`: A generic wrapper for a PyErr error
///     Parameters:
///     - `err`: A PyErr
///
#[derive(Error, Debug)]
pub enum IntegrityError {
    #[error("The plugin at path {} was not found.", .path)]
    FileNotFound { path: String },

    #[error("Missing function in plugin: {}", .func_name)]
    MissingFunction { func_name: String },

    #[error("Generic Pyo3 error during integrity check: {}", .err)]
    PyErr { err: PyErr },
}
