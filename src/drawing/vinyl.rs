use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

use super::util::audio;

///
/// An empty struct to implement the "Vinyl" draw method on.
///
pub struct VinylMethod;

impl DrawMethod for VinylMethod {
    type DrawParameters = VinylParameters;
    
    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "vinyl"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Vinyl"
    }

    ///
    /// Generates instructions to perform the vinyl drawing method.
    /// This drawing method generates a visualisation of an audio file and draws the audio
    /// waveforms on a sheet of paper.
    /// 
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error, explaning why the drawing instructions could not be created
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &VinylParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        let offset_left = (physical_dimensions.page_width() - parameters.width) / 2.;
        let offset_top = (physical_dimensions.page_height() - parameters.height) / 2.;
        let sample_spacing = parameters.width / parameters.num_samples as f64;

        if parameters.audio_path.is_empty() {
            return Err(format!("Select an audio file"));
        }

        let sample_heights = match audio::get_sampled_waveform(&parameters.audio_path, parameters.num_samples) {
            Ok(val) => val,
            Err(err) => {
                return Err(format!("Couldn't open audio file: {}", err.to_string()).to_string());
            }
        };
        let max = sample_heights.iter().max().unwrap();
        let scalar = parameters.height / (*max as f64);

        let mut surface = DrawSurface::new(physical_dimensions);
        surface.sample_xy(offset_left, offset_top).unwrap();
        
        for sample_num in 0..parameters.num_samples {
            surface.sample_xy(offset_left + sample_num as f64 * sample_spacing, offset_top + (parameters.height / 2.) - ((sample_heights[sample_num] as f64)) * scalar).unwrap();
            surface.raise_pen(false);
            surface.sample_xy(offset_left + sample_num as f64 * sample_spacing, offset_top + (parameters.height / 2.) + ((sample_heights[sample_num] as f64)) * scalar).unwrap();
            surface.raise_pen(true);
        }
        
        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `audio_path`: The path of the audio file
/// - `width`: The horizontal margin of the drawing, in millimetres
/// - `height`: The horizontal margin of the drawing, in millimetres
/// - `num_samples`: The number of samples to take on the audio waveform
///
#[derive(Serialize, Deserialize)]
pub struct VinylParameters {
    audio_path: String,

    width: f64,
    height: f64,

    num_samples: usize,
}

impl DrawParameters for VinylParameters {}


