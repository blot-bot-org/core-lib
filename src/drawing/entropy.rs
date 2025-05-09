
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;
use noise::{NoiseFn, Perlin};


///
/// An empty struct to implement the "Entropy" draw method on.
///
pub struct EntropyMethod;

impl DrawMethod for EntropyMethod {
    type DrawParameters = EntropyParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "entropy"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Entropy"
    }

    ///
    /// TODO.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &EntropyParameters) -> Result<(Vec<u8>, f64, f64), String> {

        let mut surface = DrawSurface::new(physical_dimensions);

        let center_x = physical_dimensions.page_width() / 2.;
        let center_y = physical_dimensions.page_height() / 2.;

        let angle_step_rad = (360. / parameters.cycle_density as f64) * (std::f64::consts::PI / 180.);
        let cycle_distance = parameters.cycle_distance as f64 / 100.;
        let swirl_factor = parameters.swirl_factor / 100.;

        let perlin = Perlin::new(parameters.seed);

        for i in 0..parameters.total_steps {
            let theta = i as f64 * angle_step_rad;
            let radius = parameters.start_radius + cycle_distance * theta;

            let default_x = center_x + radius * theta.cos();
            let default_y = center_y + radius * theta.sin();
            
            let (dx, dy) = get_perlin_d(default_x, default_y, &perlin, parameters);
            let (sx, sy) = swirl_transform(default_x + dx, default_y + dy, center_x, center_y, swirl_factor, parameters.swirl_decay);

            surface.sample_xy(sx + parameters.horizontal_offset, sy + parameters.vertical_offset).unwrap();
        }
        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}

fn get_perlin_d(x: f64, y: f64, perlin: &Perlin, parameters: &EntropyParameters) -> (f64, f64) {
    let mut dx = 0.;
    let mut dy = 0.;

    dx += perlin.get([x / parameters.base_size, y / parameters.base_size]) * parameters.base_strength;
    dy += perlin.get([x / parameters.base_size + 1000., y / parameters.base_size + 1000.]) * parameters.base_strength;

    dx += perlin.get([x / parameters.mid_size, y / parameters.mid_size]) * parameters.mid_strength;
    dy += perlin.get([x / parameters.mid_size + 1000., y / parameters.mid_size + 1000.]) * parameters.mid_strength;

    dx += perlin.get([x / parameters.high_size, y / parameters.high_size]) * parameters.high_strength;
    dy += perlin.get([x / parameters.high_size + 1000., y / parameters.high_size + 1000.]) * parameters.high_strength;

    (dx, dy)
}

fn swirl_transform(x: f64, y: f64, cx: f64, cy: f64, factor: f64, decay: f64) -> (f64, f64) {
    let dx = x - cx;
    let dy = y - cy;

    let r = (dx.powi(2) + dy.powi(2)).sqrt();
    let angle = dy.atan2(dx);

    let swirl_amount = factor * (-r / decay).exp();
    let new_angle = angle + swirl_amount;

    let new_x = cx + r * (new_angle).cos();
    let new_y = cy + r * (new_angle).sin();

    (new_x, new_y)
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
///
#[derive(Serialize, Deserialize)]
pub struct EntropyParameters {
    pub start_radius: f64,
    pub cycle_distance: f64,
    pub cycle_density: usize,
    pub total_steps: usize,
    pub swirl_factor: f64,
    pub swirl_decay: f64,

    pub horizontal_offset: f64,
    pub vertical_offset: f64,

    pub seed: u32,
    pub base_size: f64,
    pub base_strength: f64,
    pub mid_size: f64,
    pub mid_strength: f64,
    pub high_size: f64,
    pub high_strength: f64,
}

impl DrawParameters for EntropyParameters {}
