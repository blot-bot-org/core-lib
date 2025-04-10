
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

pub struct LinesMethod;

impl DrawMethod for LinesMethod {
    type DrawParameters = LinesParameters;

    fn get_id(&self) -> &'static str {
        "lines"
    }

    fn get_formatted_name(&self) -> &'static str {
        "Lines"
    }

    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &LinesParameters) -> Vec<u8> {
        
        let mut surface = DrawSurface::new(0., 0., physical_dimensions);
        
        for i in 0..parameters.num_lines {
            for x in 0..=100 {
                if i % 2 == 0 {
                    let x = ((physical_dimensions.page_width() - (2. * parameters.horizontal_margin as f64)) / 100.) * x as f64 + parameters.horizontal_margin as f64;
                    surface.sample_xy(x, i as f64 * 10.);
                } else {
                    let x = physical_dimensions.page_width() - parameters.horizontal_margin as f64 - ((physical_dimensions.page_width() - (2. * parameters.horizontal_margin as f64)) / 100.) * x as f64;
                    surface.sample_xy(x, i as f64 * 10.);
                }
            }

            let (current_x, current_y) = surface.get_xy();
            for y in 0..10 {
                surface.sample_xy(current_x, current_y + y as f64);
            }
        }

        surface.current_ins
    }
}



#[derive(Serialize, Deserialize)]
pub struct LinesParameters {
    pub num_lines: u32,
    pub horizontal_margin: u32,
}

impl DrawParameters for LinesParameters {}
