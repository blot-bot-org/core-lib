

/// 
/// Computes a list of points which form a circle (or oval, depending on modifiers)
///
/// # Parameters:
/// - `num_samples`: The number of samples to make on the circle
/// - `center`: The center coordinates of the circle
/// - `radius`: The radius of the circle
/// - `transform_x`: A function run on the x value of all points, can be used for scalar transformation etc
/// - `transform_y`: A function run on the x value of all points, can be used for scalar transformation etc
/// - `theta_rot`: A rotation for the circle around it's center 
///
///
pub fn get_circle_samples(num_samples: usize, center: (f64, f64), radius: f64, transform_x: Option<&dyn Fn(f64) -> f64>, transform_y: Option<&dyn Fn(f64) -> f64>, theta_rot: f64)  -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(num_samples);

    let (cx, cy) = center;

    for i in 0..num_samples {
        let angle: f64 = (2. * std::f64::consts::PI * (i as f64)) / (num_samples as f64);

        let mut x = radius * angle.cos();
        let mut y = radius * angle.sin();

        // apply any given transformations
        if let Some(fx) = transform_x {
            x = fx(x);
        }
        if let Some(fy) = transform_y {
            y = fy(y);
        }


        // rotate points
        let cos_t = theta_rot.cos();
        let sin_t = theta_rot.sin();
        let x_rot = x * cos_t - y * sin_t;
        let y_rot = x * sin_t + y * cos_t;


        points.push((cx + x_rot, cy + y_rot));
    }

    points
}
