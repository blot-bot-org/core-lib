
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;


use super::util::heightmap::gen_terrain;

///
/// An empty struct to implement the "Islands" draw method on.
///
pub struct IslandsMethod;

impl DrawMethod for IslandsMethod {
    type DrawParameters = IslandsParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "islands"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Islands"
    }

    ///
    /// Generates instructions to perform the islands drawing method.
    /// This drawing creates a set of lines, whose height is affected by 3 layers of perlin noise.
    /// The lines are layered to create a semi-2D effect, looking similar to sane dunes.
    /// Finally, the height of the heights are given a minimum of "ocean_height" to make the dunes
    /// appear in water.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error explaining why the drawing instructions could not be generated
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &IslandsParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let vertical_offset = (physical_dimensions.page_height() - parameters.height as f64) / 2. + parameters.vertical_offset as f64;
        let horizontal_offset = (physical_dimensions.page_width() - parameters.width as f64) / 2.;

        let samples_width = parameters.sample_per_mm * parameters.width;
        let layer_height = parameters.height as f64 / parameters.layers as f64;

        let heightmap_values = gen_terrain(parameters.seed, samples_width, parameters.layers, layer_height, parameters.base_size, parameters.base_amplitude, parameters.mid_size, parameters.mid_amplitude, parameters.high_size, parameters.high_amplitude);

        let mut y_samples: Vec<Vec<f64>> = Vec::new();

        // first, transform height_map to y heights on a page
        for row_idx in 0..heightmap_values.len() {
            y_samples.push(Vec::new());
            for v in heightmap_values.get(row_idx).unwrap() {
                y_samples[row_idx].push(row_idx as f64 * layer_height + ((*v).min(u8::MAX - parameters.ocean_height) as f64 / 5.));
            }
        }


        // transform y_samples to be the max of itself and the value below it
        // obviously ignore the lowest row, and iterate upwards
        for row_idx in 1..y_samples.len() {
            let rev_row_idx = y_samples.len() - row_idx - 1; // + 1 to this to get row below

            for n in 0..y_samples[rev_row_idx].len() {
                y_samples[rev_row_idx][n] = y_samples[rev_row_idx][n].min(y_samples[rev_row_idx + 1][n]);
            }
        }

        let mut surface = DrawSurface::new(physical_dimensions);
        surface.raise_pen(false);

        for layer_idx in 0..parameters.layers {
            // go left else go right
            if layer_idx % 2 == 0 {

                for item_idx in 0..y_samples[layer_idx].len() {
                    surface.sample_xy(horizontal_offset + parameters.width as f64 * (item_idx as f64 / y_samples[layer_idx].len() as f64), vertical_offset + y_samples[layer_idx][item_idx]).unwrap();
                }

            } else {
                    
                for item_idx in 0..y_samples[layer_idx].len() {
                    surface.sample_xy(physical_dimensions.page_width() - horizontal_offset - parameters.width as f64 * (item_idx as f64 / y_samples[layer_idx].len() as f64), vertical_offset + y_samples[layer_idx][y_samples[layer_idx].len() - item_idx - 1]).unwrap();
                }

            }
        }

        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `seed`: A seed to use for the random perlin noise
/// - `layers`: The number of vertical layers
/// - `sample_per_mm`: The number of samples of perlin noise per horizontal millimetre
/// - `width`: Total width of the drawing, in millimetres
/// - `height`: Total height of the drawing, in millimetres
/// - `vertical_offset`: A y-offset for the entire drawing, in millimetres
/// - `ocean_height`: The supposed "ocean-height" around the islands
/// - `base_size`: The size of the base perlin noise
/// - `base_amplitude`: The amplitude of the mid perlin noise
/// - `mid_size`: The size of the mid perlin noise
/// - `mid_amplitude`: The amplitude of the mid perlin noise
/// - `high_size`: The size of the high perlin noise
/// - `high_amplitude`: The amplitude of the high perlin noise
///
#[derive(Serialize, Deserialize)]
pub struct IslandsParameters {
    seed: u32,

    layers: usize,
    sample_per_mm: usize,
    width: usize,
    height: usize,
    vertical_offset: isize,
    
    ocean_height: u8,
    base_size: f64,
    base_amplitude: f64,
    mid_size: f64,
    mid_amplitude: f64,
    high_size: f64,
    high_amplitude: f64,
}

impl DrawParameters for IslandsParameters {}

