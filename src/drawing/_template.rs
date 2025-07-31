
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "%%%" draw method on.
///
pub struct %%%Method;


impl DrawMethod for %%%Method {
    type DrawParameters = %%%Parameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "%%%TOLOWERCASE"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "%%%"
    }

    ///
    /// Generates instructions to perform the %%%TOLOWERCASE drawing method.
    /// This drawing method .........................................
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &%%%Parameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let mut surface = DrawSurface::new(physical_dimensions);
        
        surface.sample_xy(10., 10.).unwrap();
        surface.raise_pen(false);
        surface.sample_xy(20., 20.).unwrap();
        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// ............................................
///
#[derive(Serialize, Deserialize)]
pub struct %%%Parameters {
}

impl DrawParameters for %%%Parameters {}
