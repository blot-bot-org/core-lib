
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;
use crate::drawing::util::*;

///
/// An empty struct to implement the "Bubbles" draw method on.
///
pub struct BubblesMethod;

impl DrawMethod for BubblesMethod {
    type DrawParameters = BubblesParameters;

    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "bubbles"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Bubbles"
    }

    ///
    /// Generates instructions to perform the bubbles drawing method.
    /// This drawing method uses a weighted voronoi stippling technique in order to create an even
    /// distribution of points on a plane. Finally, it creates circles in conjunction with these
    /// points to simulate bubbles.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error explaining why the drawing instructions could not be generated
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &BubblesParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        if parameters.image_path.is_empty() {
            return Err("Select an input image".to_owned());
        }

        let relaxation_coefficient = parameters.relaxation_tendency as f32 / 100.;
        
        let stippled_points: Vec<stipple_structures::Point> = match stipple::stipple_points(parameters.image_path.as_str(), parameters.num_stipples, parameters.num_iterations, relaxation_coefficient, parameters.brightness_threshold) {
            Ok(val) => val,
            Err(err_str) => return Err(err_str),
        };
        let tour = stipple::nearest_neighbour_tour(&stippled_points);

        let max_x = stippled_points.iter().max_by_key(|p| p.x).unwrap().x.into_inner();
        let max_y = stippled_points.iter().max_by_key(|p| p.y).unwrap().y.into_inner();

        let biggest_divisor = 1. / (parameters.width / max_x).min(parameters.height / max_y);

        let mut surface = DrawSurface::new(physical_dimensions);
        for t in tour.windows(2) {
            let scaled_x = stippled_points[t[0]].x.into_inner() / biggest_divisor;
            let scaled_y = stippled_points[t[0]].y.into_inner() / biggest_divisor;
            
            let center: (f32, f32) = ((scaled_x + stippled_points[t[1]].x.into_inner() / biggest_divisor) / 2., (scaled_y + stippled_points[t[1]].y.into_inner() / biggest_divisor) / 2.);
            let dist_to_next = ((stippled_points[t[1]].x / biggest_divisor - scaled_x).powi(2) + (stippled_points[t[1]].y / biggest_divisor - scaled_y).powi(2)).sqrt();

            let start_angle = (scaled_y - center.1).atan2(scaled_x - center.0);
            let end_angle = start_angle + 1.5 * 2. * std::f32::consts::PI;

            let radius = dist_to_next / 2.;
            let iterations: usize = ((radius * 6.) as usize).max(6);

            for i in 0..=iterations {
                let theta = start_angle + (end_angle - start_angle) * (i as f32 / iterations as f32);
                let offset_x = f32::cos(theta) * radius;
                let offset_y = f32::sin(theta) * radius;
                // let lerped = lerp_xy(scaled_x + offset_x, stippled_points[t[1]].x / 5., scaled_y + offset_y, stippled_points[t[1]].y / 5., (i as f32 / iterations as f32));

                // surface.sample_xy((scaled_x + offset_x + lerped.0).into_inner() as f64, (scaled_y + offset_y + lerped.1).into_inner() as f64);
                if let Err(err_str) = surface.sample_xy((center.0 + offset_x + parameters.horizontal_offset) as f64, (center.1 + offset_y + parameters.vertical_offset) as f64) {
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
/// - `image_path`: The path of the image to stipple
/// - `width`: The maximum width of the drawing
/// - `height`: The maximum height of the drawing
/// - `horizontal_offset`: The horizontal offset of the drawing
/// - `vertical_offset`: The vertical offset of the drawing
/// - `num_stipples`: The desired number of stipple points
/// - `num_iterations`: The desired number of iterations of Lloyd's relaxation
/// - `relaxation_tendency`: A float to represent a scalar multiplier for the relaxation tendency
///
#[derive(Serialize, Deserialize)]
pub struct BubblesParameters {
    image_path: String,

    width: f32,
    height: f32,
    horizontal_offset: f32,
    vertical_offset: f32,

    brightness_threshold: u8,

    num_stipples: usize,
    num_iterations: usize,
    relaxation_tendency: u8,
}

impl DrawParameters for BubblesParameters {}
