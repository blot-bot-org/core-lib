
use crate::drawing::{DrawMethod, DrawParameters};
use crate::hardware::PhysicalDimensions;
use rand::seq::index::sample;
use serde::{Serialize, Deserialize};
use crate::drawing::DrawSurface;
use crate::drawing::util::*;

pub struct ScribbleMethod;

impl DrawMethod for ScribbleMethod {
    type DrawParameters = ScribbleParameters;

    fn get_id(&self) -> &'static str {
        "scribble"
    }

    fn get_formatted_name(&self) -> &'static str {
        "Scribbles"
    }

    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, parameters: &ScribbleParameters) -> Vec<u8> {
        
        let mut surface = DrawSurface::new(0., 0., physical_dimensions);
        
        let stippled_points: Vec<stipple_structures::Point> = stipple::stipple_points("./input.jpeg", 4000, 20, 0.02);
        let tour = stipple::nearest_neighbour_tour(&stippled_points);
        println!("Finished tour generation!");

        for t in tour.windows(2) {
            let scaled_x = stippled_points[t[0]].x / 5.;
            let scaled_y = stippled_points[t[0]].y / 5.;
            
            let dist_to_next = ((stippled_points[t[1]].x / 5. - stippled_points[t[0]].x / 5.).powi(2) + (stippled_points[t[1]].y / 5. - stippled_points[t[0]].y / 5.).powi(2)).sqrt();

            let radius = dist_to_next / 5.;
            for i in 0..10 {
                let offset_x = f32::sin((i as f32 * (std::f32::consts::PI)) / 10.) * radius;
                let offset_y = f32::cos((i as f32 * (std::f32::consts::PI)) / 10.) * radius;

                surface.sample_xy((scaled_x + offset_x).into_inner() as f64, (scaled_y + offset_y as f32).into_inner() as f64);
            }
        }

        surface.current_ins
    }
}


#[derive(Serialize, Deserialize)]
pub struct ScribbleParameters {
    num_stipples: usize,
    num_iterations: usize,
    relaxation_tendency: f32
}

impl DrawParameters for ScribbleParameters {}
