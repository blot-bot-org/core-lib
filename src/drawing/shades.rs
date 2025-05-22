use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Shades" draw method on.
///
pub struct ShadesMethod;

impl DrawMethod for ShadesMethod {
    type DrawParameters = ShadesParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "shades"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Shades"
    }

    ///
    /// This drawing methods creates lines that converge into each other, within a box.
    /// It is the first / test drawing method for the pen lifting off the page.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &ShadesParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let offset_left = (physical_dimensions.page_width() - parameters.width) / 2.;
        let offset_top = (physical_dimensions.page_height() - parameters.height) / 2.;

        let mut heights = Vec::with_capacity(parameters.num_lines);
        for i in 0..parameters.num_lines {
            heights.push((parameters.height) * ( (i as f64 / parameters.num_lines as f64).powf(parameters.power as f64 / 10.) ));
        }
    
        let mut surface = DrawSurface::new(physical_dimensions);

        surface.sample_xy(offset_left, offset_top).unwrap();
        
        for i in 0..parameters.num_lines {
            surface.sample_xy(offset_left, offset_top + heights[i]).unwrap();
            surface.raise_pen(false);
            surface.sample_xy(offset_left + parameters.width, offset_top + heights[i]).unwrap();
            surface.raise_pen(true);
        }
        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `width`: The horizontal margin of the drawing, in millimetres
/// - `height`: The horizontal margin of the drawing, in millimetres
/// - `num_lines`: The number of horizontal lines to draw
/// - `power`: The tendency for the lines to converge
///
#[derive(Serialize, Deserialize)]
pub struct ShadesParameters {
    pub width: f64,
    pub height: f64,

    pub num_lines: usize,
    pub power: usize,
}

impl DrawParameters for ShadesParameters {}

