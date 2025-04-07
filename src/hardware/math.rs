
///
/// Converts cartesian into belt lengths. The calculated belt lengths are
/// relative to the motor shaft. All values are in millimetres.
///
/// # Parameters:
/// - `x`: The x parameter of the cartesian coordinate, horizontally relative to the left motor
/// - `y`: The y parameter of the cartesian coordinate, vertically relative to the left motor
/// - `motor_interspace`: The distance between the two motor shafts
///
/// # Returns:
/// - A tuple containing the left and right belt lengths, respectively
///
pub fn cartesian_to_belt(x: f64, y: f64, motor_interspace: f64) -> (f64, f64) {
    let left_belt = f64::sqrt(f64::powi(x, 2) + f64::powi(y, 2));
    let right_belt = f64::sqrt(f64::powi(motor_interspace - x, 2) + f64::powi(y, 2));

    (left_belt, right_belt)
}

///
/// Converts belt lengths into cartesian coordinates. The calculated cartesian coordinates are
/// relative to the left motor shaft (0, 0), and grow downwards/rightwards. All values are in
/// millimetres.
///
/// # Parameters:
/// - `left_belt`: The length of the left motor belt, relative to the left motor shaft
/// - `right_belt`: The length of the right motor belt, relative to the right motor shaft
/// - `motor_interspace`: The distance between the two motor shafts
/// 
/// # Returns:
/// - A tuple containing the x and y coordinates, respectively
///
pub fn belt_to_cartesian(left_length: f64, right_length: f64, motor_interspace: f64) -> (f64, f64) {
    let x = (f64::powi(motor_interspace, 2) + f64::powi(left_length, 2) - f64::powi(right_length, 2)) / (2. * motor_interspace);
    let y = f64::sqrt(f64::powi(left_length, 2) - f64::powi(x, 2));

    return (x, y);
}


///
/// Calculates the number of steps required to move the belt one millimetre.
///
/// # Returns:
/// - The required number of steps for the belt to move 1 millimetre
///
pub fn steps_per_mm() -> f64 {
    /// The number of motor steps required for one revolution.
    const STEPS_PER_REV: f64 = 3200.;
    /// The diameter of the pulley wheel, in millimetres.
    const WHEEL_DIAMETER: f64 = 12.63;

    STEPS_PER_REV / (std::f64::consts::PI * WHEEL_DIAMETER)
}

///
/// Calculates the number of millimetres moved, provided a given amount of steps.
///
/// # Parameters:
/// - `steps`: The number of proposed steps
///
/// # Returns:
/// - The number of millimetres moved
///
pub fn steps_to_mm(steps: i16) -> f64 {
    (steps as f64) / steps_per_mm()
}
