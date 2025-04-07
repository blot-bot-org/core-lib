use crate::hardware::math::*;

///
/// A structure representing the real world belts. 
///
/// # Fields:
/// - `left_belt_length`: The distance between the left motor shaft and the pen, in millimetres.
/// - `right_belt_length`: The distance between the right motor shaft and the pen, in millimetres.
/// - `motor_interspace`: The distance (horizontal) between the two motor shafts, in millimetres.
///
pub struct Belts {
    left_belt_length: f64,
    right_belt_length: f64,
    motor_interspace: f64
}

impl Belts {
    /// 
    /// Initialises a new belt object, by belt lengths.
    ///
    /// # Parameters:
    /// - `left_belt_length`: The initial left belt length, between the left motor shaft and pen
    /// - `right_belt_length`: The initial right belt length, between the left motor shaft and pen
    /// - `motor_interspace`: The distance (horizontal) between the two motor shafts
    ///
    /// # Returns:
    /// - A new `Belts` instance
    ///
    pub fn new_by_length(left_belt_length: f64, right_belt_length: f64, motor_interspace: f64) -> Belts {
        Belts { left_belt_length, right_belt_length, motor_interspace }
    }

    /// 
    /// Initialises a new belt object, by cartesian coordinates.
    ///
    /// # Parameters:
    /// - `canvas_x`: The initial x coordinate of the pen, relative to the left motor shaft 
    /// - `canvas_y`: The initial y coordinate of the pen, relative to the left motor shaft 
    /// - `motor_interspace`: The distance (horizontal) between the two motor shafts
    ///
    /// # Returns:
    /// - A new `Belts` instance
    ///
    pub fn new_by_cartesian(canvas_x: f64, canvas_y: f64, motor_interspace: f64) -> Belts {
        let (left_belt_length, right_belt_length) = cartesian_to_belt(canvas_x, canvas_y, motor_interspace);
        Self {left_belt_length, right_belt_length, motor_interspace }
    }

    ///
    /// Performs a movement of the left belt, given an amount of steps.
    ///
    /// # Parameters:
    /// - `steps`: The number of steps to move, can be negative
    ///
    fn move_left(&mut self, steps: i16) {
        self.left_belt_length += steps_to_mm(steps);
    }

    ///
    /// Performs a movement of the right belt, given an amount of steps.
    ///
    /// # Parameters:
    /// - `steps`: The number of steps to move, can be negative
    ///
    fn move_right(&mut self, steps: i16) {
        self.right_belt_length += steps_to_mm(steps);
    }

    ///
    /// Performs a movement of both belts, given an amount of steps.
    ///
    /// # Parameters:
    /// - `left_steps`: The number of steps to move the left belt, can be negative
    /// - `right_steps`: The number of steps to move the right belt, can be negative
    ///
    pub fn move_by_steps(&mut self, left_steps: i16, right_steps: i16) {
        self.move_left(left_steps);
        self.move_right(right_steps);
    }

    ///
    /// Gets the cartesian coordinates of the pen, given the current belt lengths. The cartesian
    /// coordinates are relative to the top left motor shaft, in millimetres. 
    ///
    /// # Returns:
    /// - The (x, y) coordinates of the current pen position
    ///
    pub fn get_as_cartesian(&self) -> (f64, f64) {
        belt_to_cartesian(self.left_belt_length, self.right_belt_length, self.motor_interspace)
    }

    ///
    /// # Returns:
    /// - The left and right belt lengths, respectively
    ///
    pub fn get_lengths(&self) -> (f64, f64) {
        (self.left_belt_length, self.right_belt_length)
    }
}
