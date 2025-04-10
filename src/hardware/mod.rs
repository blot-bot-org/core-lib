//!
//! Physical hardware representations and handling
//! 

pub mod math;

///
/// A simple container for the physical dimensions of the machine layout.
/// All fields are measured in millimetres.
/// All features have an associated getter function.
///
/// # Fields:
/// - `motor_distance`: The horizontal distance between the motors
/// - `page_horizontal_offset`: The horizontal distance between the left motor shaft and the top left of the page
/// - `page_vertical_offset`: The vertical distance between the left motor shaft and the top left of the page
/// - `page_width`: The width of the page
/// - `page_height`: The height of the page
/// 
#[derive(getset::Getters)]
#[get = "pub"]
pub struct PhysicalDimensions {
    motor_interspace: f64,
    page_horizontal_offset: f64,
    page_vertical_offset: f64,
    page_width: f64,
    page_height: f64
}

impl PhysicalDimensions {
    ///
    /// A function to create a new PhysicalDimension object.
    /// Ideally, this is a singleton which is reconstructed when the user changes the parameters in the frontend.
    ///
    /// # Returns:
    /// - A new `PhysicalDimension` instance
    pub fn new(motor_interspace: f64, page_horizontal_offset: f64, page_vertical_offset: f64, page_width: f64, page_height: f64) -> PhysicalDimensions {
        PhysicalDimensions { motor_interspace, page_horizontal_offset, page_vertical_offset, page_width, page_height }
    }
}
