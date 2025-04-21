
use std::default;

use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use rand::seq::index::sample;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

pub struct CascadeMethod;

impl DrawMethod for CascadeMethod {
    type DrawParameters = CascadeParameters;

    fn get_id(&self) -> &'static str {
        "cascade"
    }

    fn get_formatted_name(&self) -> &'static str {
        "Cascade"
    }

    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &CascadeParameters) -> Vec<u8> {
        
        let mut surface = DrawSurface::new(0., 0., physical_dimensions);

        
        let vertical_mm_per_box = (physical_dimensions.page_height() - 2. * parameters.vertical_margin) / parameters.boxes_vertical as f64;
        let horizontal_mm_per_box = (physical_dimensions.page_width() - 2. * parameters.horizontal_margin) / parameters.boxes_horizontal as f64;
        let line_sample: usize = 16;
        let mm_per_sample = vertical_mm_per_box / line_sample as f64;

        let mut triangle_pattern: Vec<Vec<usize>> = Vec::with_capacity(parameters.boxes_horizontal);
        for i in 0..parameters.boxes_horizontal {
            triangle_pattern.push(Vec::new());

            let total_singles = (parameters.boxes_vertical as f64 / 2.5).round() as usize;
            let mut others = parameters.boxes_vertical - total_singles;

            for _ in 0..total_singles {
                if rand::rng().random::<f32>() < 0.8 {
                    triangle_pattern[i].push(1);
                } else {
                    triangle_pattern[i].push(1);
                }
            }

            if i % 3 == 0 {

            for _ in 0..others {
                let rand_num = (rand::rng().random::<f64>() * 20.).round() as usize + 10;

                if rand_num >= others {
                    triangle_pattern[i].push(others);
                    break;
                }

                triangle_pattern[i].push(rand_num);
                others -= rand_num;
            }
            } else {
            for _ in 0..others {
                triangle_pattern[i].push(1);
            }
            }
            
            triangle_pattern[i].shuffle(&mut rand::rng());
        }

        surface.sample_xy(parameters.horizontal_margin, parameters.vertical_margin);


        for h in 0..parameters.boxes_horizontal {
            for (idx, v) in triangle_pattern[h].iter().enumerate() {

                let center_x = parameters.horizontal_margin + h as f64 * horizontal_mm_per_box;
                // sum total triangles to this point, assuming a 0 triangle is of height 1
                let going_down_y = triangle_pattern[h][0..idx]
                    .iter()
                    .map(|x|
                        match *x == 0 {
                            true => 1,
                            false => *x
                        })
                    .sum::<usize>();
                
                if h % 2 == 0 {
                    
                    self.triangle(&mut surface, center_x, parameters.vertical_margin + going_down_y as f64 * vertical_mm_per_box, horizontal_mm_per_box, line_sample * v, mm_per_sample);

                } else {

                    self.triangle_from_bottom(&mut surface, center_x, physical_dimensions.page_height() - parameters.vertical_margin - going_down_y as f64 * vertical_mm_per_box, horizontal_mm_per_box, line_sample * v, mm_per_sample);
                }

            }
        }


        surface.current_ins
    }
}

impl CascadeMethod {
    fn triangle(&self, surface: &mut DrawSurface, center_x: f64, start_y: f64, max_width: f64, samples: usize, step_height: f64) -> (f64, f64) {
        for s in 0..samples {
            surface.sample_xy(center_x, start_y + step_height * s as f64);
            surface.sample_xy(center_x - max_width * 0.5 * (1. - s as f64 / samples as f64), start_y + step_height * s as f64);
            surface.sample_xy(center_x + max_width * 0.5 * (1. - s as f64 / samples as f64), start_y + step_height * s as f64);
            surface.sample_xy(center_x, start_y + step_height * s as f64);
        }

        (center_x, start_y + step_height * (samples - 1) as f64)
    }

    fn triangle_from_bottom(&self, surface: &mut DrawSurface, center_x: f64, start_y: f64, max_width: f64, samples: usize, step_height: f64) {
        for s in 0..samples {
            surface.sample_xy(center_x, start_y - step_height * s as f64);
            surface.sample_xy(center_x - max_width * 0.5 * (s as f64 / samples as f64), start_y - step_height * s as f64);
            surface.sample_xy(center_x + max_width * 0.5 * (s as f64 / samples as f64), start_y - step_height * s as f64);
            surface.sample_xy(center_x, start_y - step_height * s as f64);
        }
    }
}



#[derive(Serialize, Deserialize)]
pub struct CascadeParameters {
    pub horizontal_margin: f64,
    pub vertical_margin: f64,

    pub boxes_vertical: usize,
    pub boxes_horizontal: usize,
}

impl DrawParameters for CascadeParameters {}
