use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use image::{GrayImage, ImageReader, Luma};
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Waves" draw method on.
///
pub struct WavesMethod;

impl DrawMethod for WavesMethod {
    type DrawParameters = WavesParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "waves"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Waves"
    }

    ///
    /// Generates instructions to perform the waves drawing method.
    /// This drawing method generates layers of sine waves, which are more intense
    /// in darker areas of the input image.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &WavesParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let input_image = match ImageReader::open("./input.jpeg") {
            Ok(img) => {
                img.decode().unwrap().into_rgb8()
            },
            Err(err) => {
                return Err(format!("Error loading image. {}", err.to_string()).to_owned());
            }
        };
        let mut surface = DrawSurface::new(physical_dimensions);

        let temp_max_width = physical_dimensions.page_width() - (parameters.horizontal_margin as f64 * 2.);
        let temp_max_height = physical_dimensions.page_height() - (parameters.vertical_margin as f64 * 2.);
        let divisor = 1. / (temp_max_width / input_image.width() as f64).min(temp_max_height / input_image.height() as f64);

        let total_width = input_image.width() as f64 / divisor;
        let total_height = input_image.height() as f64 / divisor;

        let height_per_wave = total_height / parameters.num_waves as f64;
        let mm_per_x_sample = total_width / parameters.horizontal_samples as f64;
        let wave_multiplier = parameters.wave_amplifier / 10.;

        let true_horizontal_margin = (*physical_dimensions.page_width() - total_width) / 2.;
        let true_vertical_margin = (*physical_dimensions.page_height() - total_height) / 2.;

        // we will approximate the image to the dedicated size + make it greyscale
        let mut processed_img = GrayImage::new(parameters.horizontal_samples as u32, parameters.num_waves as u32);
        for x in 0..parameters.horizontal_samples {
            for y in 0..parameters.num_waves {

                let pix = input_image.get_pixel(((input_image.width() as f64 * (x as f64 / parameters.horizontal_samples as f64)).round() as u32).min(input_image.width() - 1) as u32, (((input_image.height() as f64 * (y as f64 / parameters.num_waves as f64)).round() as u32).min(input_image.height() - 1)) as u32).0;
                let mean = (((pix[0] as f32 * 0.299 + pix[1] as f32 * 0.587 + pix[2] as f32 * 0.114)).round() as u8);
                *processed_img.get_pixel_mut(x as u32, y as u32) = Luma([mean as u8]);

            }
        }

        // processed_img.save("hi.jpg");

        for row_idx in 0..parameters.num_waves {
            for sample_idx in 0..parameters.horizontal_samples {
                let is_reversed = row_idx % 2 == 1;
                
                let iterations = 10;
                let step_x = mm_per_x_sample / iterations as f64;
                let start_x = match is_reversed {
                    false => true_horizontal_margin + sample_idx as f64 * mm_per_x_sample,
                    true => *physical_dimensions.page_width() as f64 - true_horizontal_margin - (sample_idx as f64 * mm_per_x_sample),
                };
                let start_y = true_vertical_margin as f64 + row_idx as f64 * height_per_wave + 0.5 * height_per_wave;

                let intensity = 1. - (processed_img.get_pixel(if is_reversed { (parameters.horizontal_samples - sample_idx - 1) } else { sample_idx } as u32, row_idx as u32).0[0] as f64) / 255.;

                for i in 0..iterations {
                    if is_reversed {
                        surface.sample_xy(start_x - (i + 1) as f64 * step_x, start_y + (i as f64).sin() * intensity * wave_multiplier).unwrap();
                    } else {
                        surface.sample_xy(start_x + i as f64 * step_x, start_y + (i as f64).sin() * intensity * wave_multiplier).unwrap();
                    }
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
/// - `num_waves`: The number of vertical lines to draw
/// - `horizontal_margin`: The horizontal margin of the drawing, in millimetres
///
#[derive(Serialize, Deserialize)]
pub struct WavesParameters {
    pub num_waves: usize,
    pub horizontal_samples: usize,

    pub horizontal_margin: u32,
    pub vertical_margin: u32,

    pub wave_amplifier: f64,
}

impl DrawParameters for WavesParameters {}
