use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Links" draw method on.
///
pub struct LinksMethod;

impl DrawMethod for LinksMethod {
    type DrawParameters = LinksParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "links"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Links"
    }

    ///
    /// This drawing method seeds points, before performing a Dijkstra's to create interesting
    /// patterns.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &LinksParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let mut surface = DrawSurface::new(physical_dimensions);

        let offset_x = (physical_dimensions.page_width() - parameters.width) / 2.;
        let offset_y = (physical_dimensions.page_height() - parameters.height) / 2.;

        // generate points
        let mut seeds: Vec<(f64, f64)> = Vec::new();
        for _ in 0..parameters.num_points {
            let random_x = rand::rng().random::<f64>() * parameters.width;
            let random_y = rand::rng().random::<f64>() * parameters.height;

            seeds.push((random_x, random_y));
        }


        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `num_links`: The number of vertical lines to draw
/// - `horizontal_margin`: The horizontal margin of the drawing, in millimetres
///
#[derive(Serialize, Deserialize)]
pub struct LinksParameters {
    pub width: f64,
    pub height: f64,

    pub num_points: usize,
}

impl DrawParameters for LinksParameters {}

