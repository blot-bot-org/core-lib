
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;
use crate::drawing::util::*;
use ordered_float::OrderedFloat;

///
/// An empty struct to implement the "Scribbles" draw method on.
///
pub struct ScribbleMethod;

impl DrawMethod for ScribbleMethod {
    type DrawParameters = ScribbleParameters;

    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "scribble"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Scribbles"
    }

    ///
    /// Generates instructions to perform the scribbles drawing method.
    /// This drawing method uses a weighted voronoi stippling technique in order to create an even
    /// distribution of points on a plane. Finally, it creates circles in conjunction with these
    /// points to simulate scribbles.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An instruction set, represented as a u8 vector, containing the draw calls
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &ScribbleParameters) -> Vec<u8> {
        
        let mut surface = DrawSurface::new(0., 0., physical_dimensions);
        
        let stippled_points: Vec<stipple_structures::Point> = stipple::stipple_points("./input.jpeg", parameters.num_stipples, parameters.num_iterations, parameters.relaxation_tendency);
        let tour = stipple::nearest_neighbour_tour(&stippled_points);
        println!("Finished tour generation!");

        let radius_divisor = ((100 - (parameters.scribble_size)) as f32 / 100.) * 5.;

        for t in tour.windows(2) {
            let scaled_x = stippled_points[t[0]].x / 5.;
            let scaled_y = stippled_points[t[0]].y / 5.;
            
            let dist_to_next = ((stippled_points[t[1]].x / 5. - stippled_points[t[0]].x / 5.).powi(2) + (stippled_points[t[1]].y / 5. - stippled_points[t[0]].y / 5.).powi(2)).sqrt();

            let radius = dist_to_next / radius_divisor;
            let iterations: usize = ((radius * 4.) as usize).max(6);
            for i in 0..=iterations {
                let theta = 2. * std::f32::consts::PI * (i as f32 / (iterations) as f32);
                let offset_x = f32::sin(theta) * radius;
                let offset_y = f32::cos(theta) * radius;
                // let lerped = lerp_xy(scaled_x + offset_x, stippled_points[t[1]].x / 5., scaled_y + offset_y, stippled_points[t[1]].y / 5., (i as f32 / iterations as f32));

                // surface.sample_xy((scaled_x + offset_x + lerped.0).into_inner() as f64, (scaled_y + offset_y + lerped.1).into_inner() as f64);
                 surface.sample_xy((scaled_x + offset_x).into_inner() as f64, (scaled_y + offset_y + parameters.vertical_offset).into_inner() as f64);
            }
        }

        surface.current_ins
    }
}

fn lerp_xy(x1: OrderedFloat<f32>, x2: OrderedFloat<f32>, y1: OrderedFloat<f32>, y2: OrderedFloat<f32>, lerp: f32) -> (f32, f32) {
    ( (x2 - x1).into_inner() * (lerp / 1.) , (y2 - y1).into_inner() * (lerp / 1.) )
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `num_stipples`: The desired number of stipple points
/// - `num_iterations`: The desired number of iterations of Lloyd's relaxation
/// - `relaxation_tendency`: A float to represent a scalar multiplier for the relaxation tendency
///
#[derive(Serialize, Deserialize)]
pub struct ScribbleParameters {
    num_stipples: usize,
    num_iterations: usize,
    relaxation_tendency: f32,
    scribble_size: usize,
    vertical_offset: f32,
}

impl DrawParameters for ScribbleParameters {}
