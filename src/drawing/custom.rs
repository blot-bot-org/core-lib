use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use crate::plugin;
use crate::plugin::interface::{GenericInstruction, SurfaceInterface};
use pyo3::types::PyAnyMethods;
use pyo3::{PyRef, Python};
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Custom" draw method on.
///
pub struct CustomMethod;


impl DrawMethod for CustomMethod {
    type DrawParameters = CustomParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "custom"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Custom"
    }

    ///
    /// Generates instructions to perform the custom drawing method.
    /// This drawing method uses a custom Python plugin to generate a drawing.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &CustomParameters) -> Result<(Vec<u8>, f64, f64), String> {

        let rust_instructions: Vec<GenericInstruction> = match Python::with_gil(|py| {
            let module = match plugin::load_plugin_module(py, &parameters.plugin_path) {
                Ok(val) => val,
                Err(err) => { return Err(err.to_string()); }
            };

            let result = plugin::verify_plugin(&module);
            match result {
                Ok(()) => {},
                Err(err) => { return Err(err.to_string()); }
            };

            // since the module is okay, we'll go ahead with generating the drawings
            
            // make the surface interface object, this will store pythons draw calls
            let surface_interface = match pyo3::Py::new(py, SurfaceInterface::new()) {
                Ok(val) => val,
                Err(err) => {
                    return Err(format!("Failed to create surface interface object for Python: {}", err.to_string()));
                }
            };

            // deserialize frontend json to json object for python
            let param_obj = match plugin::json_loads(py, &parameters.plugin_parameters_json) {
                Ok(val) => val,
                Err(err) => {
                    return Err(format!("Error parsing frontend parameters: {}", err.to_string()));
                }
            };
            

            let gen_fn = module.getattr("run").unwrap();
            match gen_fn.call1((surface_interface.as_ref(), param_obj.as_ref(), physical_dimensions.page_width(), physical_dimensions.page_height())) {
                Ok(_) => {},
                Err(err) => {
                    println!("Error in plugin: {}", err.to_string());
                    return Err(format!("Error running `run` function in plugin: {}", err.to_string()));
                }
            };

            let surface_ref = surface_interface.as_ref().extract::<PyRef<SurfaceInterface>>(py).unwrap();
            let instructions = surface_ref.get_instructions();

            Ok(instructions.clone())
        }) {
            Ok(ins) => ins,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        
        let mut surface = DrawSurface::new(physical_dimensions);
        
        for ins in rust_instructions {
            match ins.kind.as_str() {
                // literals defined in 'plugin/interface.rs'
                "sample_xy" => {
                    surface.sample_xy(ins.x.unwrap(), ins.y.unwrap()).unwrap();
                },
                "raise_pen" => {
                    surface.raise_pen(ins.raised.unwrap());
                },
                _ => {}
            }
        }
        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `plugin_path`: The path to the Python plugin file
/// - `plugin_parameters_json`: The serialized (as a string) JSON object containing the parameters
///    the plugin requires
///
#[derive(Serialize, Deserialize)]
pub struct CustomParameters {
    pub plugin_path: String,
    pub plugin_parameters_json: String
}

impl DrawParameters for CustomParameters {}
