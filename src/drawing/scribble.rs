
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



        
        // making the circular scribbly bits
        /*
        let mut last: Option<usize> = None;

        for i in 0..stippled_points.len() {
            if last.is_none() {
                last = Some(i);
                continue;
            }

            let radius = ((stippled_points[i].x - stippled_points[last.unwrap()].x).powi(2) + (stippled_points[i].y - stippled_points[last.unwrap()].y).powi(2)).sqrt() / 520.;
            let mut last_x = ordered_float::OrderedFloat(0.);
            let mut last_y = ordered_float::OrderedFloat(0.);
            let iterations: usize = (4. * radius).round() as usize;

            for j in 0..iterations.max(1) {
                let mid_x = (stippled_points[last.unwrap()].x + (stippled_points[i].x - stippled_points[last.unwrap()].x) * (j as f32 / iterations.max(1) as f32));
                let mid_y = (stippled_points[last.unwrap()].y + (stippled_points[i].y - stippled_points[last.unwrap()].y) * (j as f32 / iterations.max(1) as f32));

                let offset_x = (f32::sin((j as f32 * (std::f32::consts::PI * 2.)) / iterations.max(1) as f32)) * radius;
                let offset_y = (f32::cos((j as f32 * (std::f32::consts::PI * 2.)) / iterations.max(1) as f32)) * radius;

                let sample_x = mid_x + offset_x;
                let sample_y = mid_y + offset_y;

                last_x = sample_x;
                last_y = sample_y;

                surface.sample_xy(sample_x.into_inner() as f64, sample_y.into_inner() as f64);
            }

            if i + 2 < stippled_points.len() {
                let next_radius = ((stippled_points[i + 1].x - stippled_points[i].x).powi(2) + (stippled_points[i + 1].y - stippled_points[i].y).powi(2)).sqrt() / 3.;
                let next_mid_x = stippled_points[i].x;
                let next_mid_y = stippled_points[i].y;

                let offset_x = (f32::sin(0.) / iterations as f32) * next_radius;
                let offset_y = (f32::cos(0.) / iterations as f32) * next_radius;

                for i in 0..50 {
                    let lerp_x = (last_x + (next_mid_x + offset_x - last_x) * (i as f32 / 50.));
                    let lerp_y = (last_y + (next_mid_y + offset_y - last_y) * (i as f32 / 50.));

                    surface.sample_xy(lerp_x.into_inner() as f64, lerp_y.into_inner() as f64);
                }
            }

            last = Some(i);
        }
        */



        
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
