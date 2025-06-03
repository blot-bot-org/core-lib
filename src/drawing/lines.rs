
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Lines" draw method on.
///
pub struct LinesMethod;

impl DrawMethod for LinesMethod {
    type DrawParameters = LinesParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "lines"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Lines"
    }

    ///
    /// Generates instructions to perform the lines drawing method.
    /// This drawing method creates a set of lines which move down the page. It is used for
    /// testing, only.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &LinesParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let mut surface = DrawSurface::new(physical_dimensions);
        surface.raise_pen(false);
        
        for i in 0..parameters.num_lines {
            for x in 0..=100 {
                if i % 2 == 0 {
                    let x = ((physical_dimensions.page_width() - (2. * parameters.horizontal_margin as f64)) / 100.) * x as f64 + parameters.horizontal_margin as f64;
                    if let Err(err_str) = surface.sample_xy(x, i as f64 * 10.) {
                        return Err(err_str);
                    };
                } else {
                    let x = physical_dimensions.page_width() - parameters.horizontal_margin as f64 - ((physical_dimensions.page_width() - (2. * parameters.horizontal_margin as f64)) / 100.) * x as f64;
                    if let Err(err_str) = surface.sample_xy(x, i as f64 * 10.) {
                        return Err(err_str);
                    };
                }
            }

            let (current_x, current_y) = surface.get_xy();
            for y in 0..10 {
                if let Err(err_str) = surface.sample_xy(current_x, current_y + y as f64) {
                    return Err(err_str);
                };
            }
        }

        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `num_lines`: The number of vertical lines to draw
/// - `horizontal_margin`: The horizontal margin of the drawing, in millimetres
///
#[derive(Serialize, Deserialize)]
pub struct LinesParameters {
    pub num_lines: u32,
    pub horizontal_margin: u32,
}

impl DrawParameters for LinesParameters {}
