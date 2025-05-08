
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;

///
/// An empty struct to implement the "Cascade" draw method on.
///
pub struct CascadeMethod;

impl DrawMethod for CascadeMethod {
    type DrawParameters = CascadeParameters;

    ///
    /// # Returns:
    /// - The backend ID of the drawing method
    ///
    fn get_id(&self) -> &'static str {
        "cascade"
    }

    ///
    /// # Returns:
    /// - The frontend display name of the drawing method
    ///
    fn get_formatted_name(&self) -> &'static str {
        "Cascade"
    }

    ///
    /// Generates instructions to perform the cascade drawing method.
    /// This drawing method creates a wall of triangles falling down the page, with many single
    /// triangles, as well as a many longer triangles too.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimension object, including paper width / height
    /// - `parameters`: The user-configured parameters to adjust the drawing style
    ///
    /// # Returns:
    /// - An (instruction set, start_x, start_y), represented as a u8 vector and floats respectively
    /// - An error explainig why the drawing instructions could not be generated
    ///
    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &CascadeParameters) -> Result<(Vec<u8>, f64, f64), String> {
        
        // calculate constants
        let vertical_mm_per_box = (physical_dimensions.page_height() - 2. * parameters.vertical_margin) / parameters.boxes_vertical as f64;
        let horizontal_mm_per_box = (physical_dimensions.page_width() - 2. * parameters.horizontal_margin) / parameters.boxes_horizontal as f64;
        let line_sample: usize = 16;
        let mm_per_sample = vertical_mm_per_box / line_sample as f64;

        // the array holds a list of arrays, each containing something like [1, 1, 1, 3, 2, 1, 1, 5] where it would draw a triangle that is 1/2/3/5 "blocks" tall
        let mut triangle_pattern: Vec<Vec<usize>> = Vec::with_capacity(parameters.boxes_horizontal);

        let mut surface = DrawSurface::new(physical_dimensions);

        for i in 0..parameters.boxes_horizontal {
            triangle_pattern.push(Vec::new());

            let total_singles = (parameters.boxes_vertical as f64 / 2.5).round() as usize;
            let mut others = parameters.boxes_vertical - total_singles;

            // singles are not currently implemented.
            for _ in 0..total_singles {
                if rand::rng().random::<f32>() < 0.8 {
                    triangle_pattern[i].push(1);
                } else {
                    triangle_pattern[i].push(1);
                }
            }
            
            // only do long triangles on every 3rd row
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
            
            // shuffle them to make them appear random
            triangle_pattern[i].shuffle(&mut rand::rng());
        }

        // move to start position
        if let Err(err_str) = surface.sample_xy(parameters.horizontal_margin, parameters.vertical_margin) {
            return Err(err_str);
        };

        // for each column, draw every vertical triangle, drawing down or up
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
                    
                    if let Err(err_str) = self.triangle(&mut surface, center_x, parameters.vertical_margin + going_down_y as f64 * vertical_mm_per_box, horizontal_mm_per_box, line_sample * v, mm_per_sample) {
                        return Err(err_str);
                    };

                } else {

                    if let Err(err_str) = self.triangle_from_bottom(&mut surface, center_x, physical_dimensions.page_height() - parameters.vertical_margin - going_down_y as f64 * vertical_mm_per_box, horizontal_mm_per_box, line_sample * v, mm_per_sample) {
                        return Err(err_str);
                    };
                }

            }
        }


        Ok((surface.current_ins, surface.first_sample_x.unwrap_or(0.), surface.first_sample_y.unwrap_or(0.)))
    }
}

impl CascadeMethod {
    // I really didn't need two functions for this.

    /// 
    /// Performs a drawing of a triangle, starting from the top, on a mutable `DrawSurface`.
    ///
    /// # Parameters:
    /// - `surface`: The mutable draw surface
    /// - `center_x`: The x value of the desired center of the triangle
    /// - `start_y`: The starting y value of the triangle
    /// - `max_width`: The desired width of the triangle, at the base
    /// - `samples`: The number of samples used to draw the triangle
    /// - `step_height`: The y movement per sample, going down
    ///
    /// # Returns:
    /// - Void if the draw calls suceeded
    /// - An error as an owned string, explaining the problem
    ///
    fn triangle(&self, surface: &mut DrawSurface, center_x: f64, start_y: f64, max_width: f64, samples: usize, step_height: f64) -> Result<(), String> {
        for s in 0..samples {
            if let Err(err_str) = surface.sample_xy(center_x, start_y + step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x - max_width * 0.5 * (1. - s as f64 / samples as f64), start_y + step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x + max_width * 0.5 * (1. - s as f64 / samples as f64), start_y + step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x, start_y + step_height * s as f64) {
                return Err(err_str);
            };
        }

        Ok(())
    }

    /// 
    /// Performs a drawing of a triangle, starting from the bottom, on a mutable `DrawSurface`.
    ///
    /// # Parameters:
    /// - `surface`: The mutable draw surface
    /// - `center_x`: The x value of the desired center of the triangle
    /// - `start_y`: The starting y value of the triangle
    /// - `max_width`: The desired width of the triangle, at the base
    /// - `samples`: The number of samples used to draw the triangle
    /// - `step_height`: The y movement per sample, going up
    ///
    /// # Returns:
    /// - Void if the draw calls suceeded
    /// - An error as an owned string, explaining the problem
    ///
    fn triangle_from_bottom(&self, surface: &mut DrawSurface, center_x: f64, start_y: f64, max_width: f64, samples: usize, step_height: f64) -> Result<(), String> {
        for s in 0..samples {
            if let Err(err_str) = surface.sample_xy(center_x, start_y - step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x - max_width * 0.5 * (s as f64 / samples as f64), start_y - step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x + max_width * 0.5 * (s as f64 / samples as f64), start_y - step_height * s as f64) {
                return Err(err_str);
            };
            if let Err(err_str) = surface.sample_xy(center_x, start_y - step_height * s as f64) {
                return Err(err_str);
            };
        }
        
        Ok(())
    }
}


///
/// A set of parameters to instruct the generation of the draw calls.
///
/// # Fields:
/// - `horizontal_margin`: The horizontal margin of the drawing, in millimetres
/// - `vertical_margin`: The vertical margin of the drawing, in millimetres
/// - `boxes_horizontal`: The number of triangle columns wanted
/// - `boxes_vertical`: The number of triangle rows wanted
///
#[derive(Serialize, Deserialize)]
pub struct CascadeParameters {
    pub horizontal_margin: f64,
    pub vertical_margin: f64,

    pub boxes_vertical: usize,
    pub boxes_horizontal: usize,
}

impl DrawParameters for CascadeParameters {}
