
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;


use super::util::heightmap::gen_terrain;

///
/// An empty struct to implement the "Dunes" draw method on.
///
pub struct DunesMethod;

impl DrawMethod for DunesMethod {
    type DrawParameters = DunesParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "dunes"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Dunes"
    }

    ///
    /// Generates instructions to perform the dunes drawing method.
    /// This drawing creates a set of lines, whose height is affected by 3 layers of perlin noise.
    /// The lines are layered to create a semi-2D effect, looking similar to sane dunes.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An instruction set, represented as a u8 vector, containing the draw calls
    /// - An error explaining why the drawing instructions could not be generated
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &DunesParameters) -> Result<Vec<u8>, String> {
        
        let mut surface = DrawSurface::new(0., 0., physical_dimensions);
        let heightmap_values = gen_terrain(parameters.seed, 1000, 1000, parameters.base_size, parameters.base_amplitude, parameters.mid_size, parameters.mid_amplitude, parameters.high_size, parameters.high_amplitude);

        let mut y_samples: Vec<Vec<f64>> = Vec::new();

        // first, transform height_map to y heights on a page
        let space_between = 3.;
        for row_idx in 0..heightmap_values.len() {
            y_samples.push(Vec::new());
            for v in heightmap_values.get(row_idx).unwrap() {
                y_samples[row_idx].push(((4. - (*v as f64) * 4.) + (row_idx as f64 * space_between)) / 16.);
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

        /*
        save_debug_iter_to_file(y_samples.iter().step_by(10), "./out.txt");

        let mut img_buf: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(1000, 1000);

        for row in 0..heightmap_values.len() {
            for p in 0..heightmap_values[row].len() {
                *img_buf.get_pixel_mut(row as u32, p as u32) = Luma([heightmap_values[row][p]]);
            }
        }

        img_buf.save("./output.png");
        */


        for (it_idx, row_idx) in (0..y_samples.len()).step_by(parameters.layer_step).enumerate() {
            // go left else go right
            if it_idx % 2 == 0 {

                for item_idx in 0..y_samples[row_idx].len() {
                    surface.sample_xy(5. + item_idx as f64 / 5., parameters.vertical_offset + y_samples[row_idx][item_idx]).unwrap();
                }

            } else {
                    
                for item_idx in 0..y_samples[row_idx].len() {
                    surface.sample_xy(5. + (y_samples[row_idx].len() - item_idx) as f64 / 5., parameters.vertical_offset + y_samples[row_idx][y_samples[row_idx].len() - item_idx - 1]).unwrap();
                }

            }
        }

        Ok(surface.current_ins)
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `seed`: A seed to use for the random perlin noise
/// - `horizontal_margin`: The step size for the layer iterator
/// - `vertical_offset`: A y-offset for the entire drawing
///
/// - `base_size`: The size of the base perlin noise
/// - `base_amplitude`: The amplitude of the mid perlin noise
/// - `mid_size`: The size of the mid perlin noise
/// - `mid_amplitude`: The amplitude of the mid perlin noise
/// - `high_size`: The size of the high perlin noise
/// - `high_amplitude`: The amplitude of the high perlin noise
///
#[derive(Serialize, Deserialize)]
pub struct DunesParameters {
    seed: u32,
    layer_step: usize,
    vertical_offset: f64,

    base_size: f64,
    base_amplitude: f64,
    mid_size: f64,
    mid_amplitude: f64,
    high_size: f64,
    high_amplitude: f64,
}

impl DrawParameters for DunesParameters {}

