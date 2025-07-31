
use crate::drawing::util::geometry;
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Atom" draw method on.
///
pub struct AtomMethod;


impl DrawMethod for AtomMethod {
    type DrawParameters = AtomParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "atom"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Atom"
    }

    ///
    /// Generates instructions to perform the atom drawing method.
    /// This drawing method draws a small "nucleus" surrounded by random, orbiting shells.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &AtomParameters) -> Result<(Vec<u8>, f64, f64), String> {

        let cx = physical_dimensions.page_width() / 2.;
        let cy = physical_dimensions.page_height() / 2.;
        let scaled_scramble = parameters.nucleus_scramble / 10.;

        let mut rng = StdRng::seed_from_u64(parameters.seed);
        let mut surface = DrawSurface::new(physical_dimensions);
        surface.raise_pen(true);


        for _ in 0..parameters.num_shells {
            let x_scale;
            let y_scale;
            if rng.random_bool(0.5) {
                x_scale = 1.;
                y_scale = rng.random_range(0.2..=1.0);
            } else {
                x_scale = rng.random_range(0.2..=1.0);
                y_scale = 1.;
            }

            let radius: f64 = rng.random_range(parameters.min_shell_width..=parameters.max_shell_width);
            let theta: f64 = rng.random_range((0.)..(std::f64::consts::PI * 2.));
            
            // generate the points to draw
            let circle_points = geometry::get_circle_samples((radius.floor() as usize) * 20, (cx, cy), radius, Some(&|x| x * x_scale), Some(&|y| y * y_scale), theta);
            
            // go to the first point
            let (fx, fy) = circle_points.get(0).expect("There to be shells to draw");
            surface.sample_xy(*fx, *fy).unwrap();
            surface.raise_pen(false);

            // then draw all the points
            for (px, py) in circle_points {
                surface.sample_xy(px, py).unwrap();
            }
            surface.raise_pen(true);
        }


        // now draw the nucleus
        for _ in 0..parameters.nucleus_circles {
            // generate the points to draw
            let x_scale;
            let y_scale;
            if rng.random_bool(0.5) {
                x_scale = 1.;
                y_scale = rng.random_range(0.2..=1.0);
            } else {
                x_scale = rng.random_range(0.2..=1.0);
                y_scale = 1.;
            }

            let circle_points = geometry::get_circle_samples(
                (parameters.nucleus_size as usize) * 10,
                (cx + rng.random_range(-scaled_scramble..=scaled_scramble), cy + rng.random_range(-scaled_scramble..=scaled_scramble)),
                parameters.nucleus_size,
                Some(&|x| x * x_scale),
                Some(&|y| y * y_scale),
                rng.random_range((0.)..(std::f64::consts::PI * 2.))
            );
            
            // go to the first point
            let (fx, fy) = circle_points.get(0).expect("There to be points on the nucleus to draw");
            surface.sample_xy(*fx, *fy).unwrap();
            surface.raise_pen(false);

            // then draw all the points
            for (px, py) in circle_points {
                surface.sample_xy(px, py).unwrap();
            }
            surface.raise_pen(true);
        }

        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `num_shells`: The number of shells to draw around the nucleus
/// - `min_shell_width`: The minimum shell width
/// - `max_shell_width`: The maximum shell width
/// - `nucleus_size`: The base radius for the nucleus
/// - `nucleus_scramble`: A constant of +- randomness applied to each nucleus circle center
/// - `nucleus_circles`: The number of circles to draw to form the nucleus
///
#[derive(Serialize, Deserialize)]
pub struct AtomParameters {
    pub seed: u64,
    pub num_shells: u32,
    pub min_shell_width: f64,
    pub max_shell_width: f64,
    pub nucleus_size: f64,
    pub nucleus_scramble: f64,
    pub nucleus_circles: u32
}

impl DrawParameters for AtomParameters {}
