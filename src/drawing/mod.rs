//!
//! Drawing method trait, helpers and implementations
//!

use byteorder::{BigEndian, LittleEndian, ByteOrder};
use crate::hardware::PhysicalDimensions;
use serde::{Serialize, Deserialize};
use crate::preview::belts::Belts;
use crate::hardware::math::*;

pub mod util;

pub mod lines;
pub mod cascade;
pub mod scribble;
pub mod bubbles;
pub mod islands;
pub mod dunes;
pub mod waves;

///
/// The trait for all drawing methods to implement.
///
/// # Functions:
/// - `get_id`: Should return the unique ID of a drawing method
/// - `get_formatted_name`: Should return the formatted name of a drawing method
/// - `gen_instructions`: Should return the drawing instruction bytes as a vector and pen start position, or an error. Takes the page parameters.
///
pub trait DrawMethod {
    type DrawParameters;

    fn get_id(&self) -> &'static str;
    fn get_formatted_name(&self) -> &'static str;

    fn gen_instructions(&self, physical_dimensions: &PhysicalDimensions, params: &Self::DrawParameters) -> Result<(Vec<u8>, f64, f64), String>;
}

/// 
/// The trait for all drawing parameters to implement.
/// It requires the implementation of Serialize and Deserialize.
///
pub trait DrawParameters: Serialize + for<'d> Deserialize<'d> {}

/// 
/// An abstract surface to draw on. Methods such as goto(x, y) and sample can be
/// called to construct an image.
///
/// # Fields:
/// - `first_sample_x`: The initial x position of the pen, in millimetres from the top-left motor
/// - `first_sample_y`: The initial y position of the pen, in millimetres from the top-left motor
/// - `current_ins`: The vector containing the current instructions
/// - `physical_dimensions`: The physical parameters of the machine
/// - `belts`: An object representing the belts
///
pub struct DrawSurface<'pd> {
    first_sample_x: Option<f64>,
    first_sample_y: Option<f64>,

    current_ins: Vec<u8>,
    physical_dimensions: &'pd PhysicalDimensions,
    belts: Belts,
}

#[allow(dead_code)]
impl<'pd> DrawSurface<'pd> {
    /// 
    /// Creates a new drawing surface, intialising belts to the init_x, init_y length.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimensions object representing the current hardware
    ///
    /// # Returns:
    /// - A blank `DrawSurface` object
    ///
    fn new(physical_dimensions: &PhysicalDimensions) -> DrawSurface {
        let belts = Belts::new_by_cartesian(0., 0., 0.);

        DrawSurface { current_ins: Vec::new(), physical_dimensions, belts, first_sample_x: None, first_sample_y: None }
    }

    /// 
    /// Moves the pen to a new x, y position and instructions a line between the preview and
    /// current pen position.
    /// If there is no initial position, we set the passed x, y as the initial position and update
    /// the belts to reflect this. No instructions are added in this case.
    ///
    /// # Parameters:
    /// - `x`: The new pen x position, relative to the top left of the paper in millimetres
    /// - `y`: The new pen y position, relative to the top left of the paper in millimetres
    ///
    /// # Returns:
    /// - Void if the function suceeded
    /// - An error as an owned string, explaining the problem
    ///
    fn sample_xy(&mut self, x: f64, y: f64) -> Result<(), String> {
        if self.first_sample_x.is_none() || self.first_sample_y.is_none() {
            // here we basically initialise the object
            // the first sample marks the first point of the belts
            // it does not create any instructions

            self.first_sample_x = Some(x);
            self.first_sample_y = Some(y);

            let belts = Belts::new_by_cartesian(
                self.physical_dimensions.page_horizontal_offset() + x,
                self.physical_dimensions.page_vertical_offset() + y,
                *self.physical_dimensions.motor_interspace()
            );
            self.belts = belts;

            return Ok(());
        }

        let (new_left, new_right) = cartesian_to_belt(*self.physical_dimensions.page_horizontal_offset() + x, *self.physical_dimensions.page_vertical_offset() + y, *self.physical_dimensions.motor_interspace());

        // delta length of belts in mm
        let delta_left_length = new_left - self.belts.get_lengths().0;
        let delta_right_length = new_right - self.belts.get_lengths().1;

        let delta_left_steps = delta_left_length * steps_per_mm();
        let delta_right_steps = -(delta_right_length * steps_per_mm());

        if delta_left_steps >= i16::MAX as f64 || delta_left_steps <= i16::MIN as f64 || delta_right_steps >= i16::MAX as f64 || delta_right_steps <= i16::MIN as f64 {
            return Err(format!("Steps are outside range! Currently have {} instructions generated, with step sizes l:{} and r:{}", self.current_ins.len(), delta_left_steps, delta_right_steps).to_owned());
            // TODO: Error impl
        }
        
        let ls: i16 = (delta_left_steps.round() as i16).try_into().unwrap();
        let rs: i16 = (delta_right_steps.round() as i16).try_into().unwrap();
        self.belts.move_by_steps(ls, -rs); // adjust state of belts, we have to invert the already inverted r
        // print!("{},{},", ls, rs);
    
        // prepare bytes for socket
        let mut left_step_bytes: [u8; 2] = [0_u8; 2];
        let mut right_step_bytes: [u8; 2] = [0_u8; 2];
        BigEndian::write_i16(&mut left_step_bytes, ls);
        BigEndian::write_i16(&mut right_step_bytes, rs);

        // push instruction bytes to buffer
        self.current_ins.push(left_step_bytes[0]);    
        self.current_ins.push(left_step_bytes[1]);    
        self.current_ins.push(right_step_bytes[0]);    
        self.current_ins.push(right_step_bytes[1]);    
        self.current_ins.push(0x0C_u8);

        Ok(())
    }

    /// 
    /// Pops the last draw call off the instruction list, and reverts the belts to their old
    /// position accordingly.
    ///
    /// # Returns:
    /// - An error as an owned string, if the function failed
    ///
    fn pop_sample(&mut self) -> Result<(), String> {
        if self.current_ins.len() < 5 {
            return Err("Could not pop instructions, as there were no instructions in the vector.".to_owned());
        }

        let _ = self.current_ins.pop().unwrap(); // 0x0C terminator byte

        // right step bytes popped off first
        let right_step_bytes: [u8; 2] = [self.current_ins.pop().unwrap(), self.current_ins.pop().unwrap()];
        let left_step_bytes: [u8; 2] = [self.current_ins.pop().unwrap(), self.current_ins.pop().unwrap()];

        // here we use little endian to decode, as the bytes are popped off in reverse
        let left_steps: i16 = LittleEndian::read_i16(&left_step_bytes);
        let right_steps: i16 = LittleEndian::read_i16(&right_step_bytes);

        // move the belts in reverse, to revert the instruction. right belt is double-reversed hence not
        self.belts.move_by_steps(-left_steps, right_steps);

        Ok(())
    }

    ///
    /// # Returns:
    /// - The curent (x, y) position of the pen, relative to the top corner of the paper
    /// 
    fn get_xy(&self) -> (f64, f64) {
        let (total_x, total_y) = self.belts.get_as_cartesian();
        (total_x - self.physical_dimensions.page_horizontal_offset(), total_y - self.physical_dimensions.page_vertical_offset())
    }

    /// 
    /// TODO: Lerp between 0, 0 -> init_x, init_y appropriately to fit ins into i16 bytes
    ///
    /// Creates the drawing instructions required to move the pen from 0, 0 on the page to the
    /// given point, used to position the pen initially to start the drawing.
    ///
    /// # Parameters:
    /// - `physical_dimensions`: A physical dimensions object representing the current hardware
    /// - `init_x`: The target x position to move to
    /// - `init_y`: The target x position to move to
    ///
    /// # Returns:
    /// - A vector of instruction bytes
    ///
    pub fn pen_to_start_ins(physical_dimensions: &PhysicalDimensions, init_x: f64, init_y: f64) -> Vec<u8> {
        let mut ds = DrawSurface::new(physical_dimensions);

        ds.sample_xy(0., 0.).unwrap(); // init at to top/left of page
        ds.sample_xy(init_x, init_y).unwrap(); // move to start pos
        
        ds.current_ins
    }
}
