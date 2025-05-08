use noise::{NoiseFn, PerlinSurflet};

/// 
/// Generates a 2D vector of perlin noise, with an f64 between 0 and 1 to represent height.
///
/// # Parameters:
/// - `seed`: A number to seed the perlin noise
/// - `width`: The number of horizontal samples
/// - `height`: The number of vertical samples
/// - `layer_height`: The y step-size per layer
/// - `base_scale`: The scale of the perlin noise, between 1 and 255
///
/// # Returns:
/// - A 2D vector, with f64 values, in the form Row<Column<f64>>
///
fn gen_heightmap(seed: u32, width: usize, height: usize, layer_height: f64, base_scale: f64) -> Vec<Vec<f64>> {
    let perlin = PerlinSurflet::new(seed);

    let base_scale_coefficient = 255. / base_scale;

    let mut vec: Vec<Vec<f64>> = vec![];
    for i in 0..height {
        let mut v = vec![];
        for j in 0..width {
            v.push(perlin.get( [ (j as f64 / width as f64) * base_scale_coefficient, ((i as f64 * layer_height) / height as f64) * base_scale_coefficient ] ));
        }
        vec.push(v);
    }

    vec
}


/// 
/// Generates "terrain", an alias for 3 layers of perlin noise.
/// In short, it layers the `gen_heightmap` function a couple of times.
///
/// # Parameters:
/// - `seed`: A number to seed the perlin noise
/// - `width`: The number of horizontal samples
/// - `height`: The number of vertical samples
/// - `layer_height`: The y step-size per layer
/// - `bs` + `ba`: Perlin noise layer 1's size and amplitude
/// - `ms` + `ma`: Perlin noise layer 2's size and amplitude
/// - `hs` + `ha`: Perlin noise layer 3's size and amplitude
///
/// # Returns:
/// - A 2D vector, with u8 values, in the form Row<Column<u8>>
///
pub fn gen_terrain(seed: u32, width: usize, height: usize, layer_height: f64, bs: f64, ba: f64, ms: f64, ma: f64, hs: f64, ha: f64) -> Vec<Vec<u8>> {
    let mut values: Vec<Vec<u8>> = Vec::new();
    for n in 0..height {
        values.push(Vec::new());
        for _ in 0..width {
            values[n].push(0);
        }
    }
    
    let base_vals: Vec<Vec<f64>> = gen_heightmap(seed * 2, width, height, layer_height, bs);

    for row in 0..base_vals.len() {
        for pix in 0..base_vals[row].len() {
            *values.get_mut(row).unwrap().get_mut(pix).unwrap() = (((255. - ((*base_vals.get(row).unwrap().get(pix).unwrap() as f64 + 1.) / 2. * ba).round() as f64) / 255.).powi(2) * 255. ).min(255.).max(0.) as u8;
        }
    }

    let mid_vals: Vec<Vec<f64>> = gen_heightmap(seed * 4, width, height, layer_height, ms);

    for row in 0..mid_vals.len() {
        for pix in 0..mid_vals[row].len() {
            let existing_luma = values.get(row).unwrap().get(pix).unwrap();
            //               this here could have mid-cutoff value
            let sub_value = (((255 - *existing_luma) as f64 / 255.) * (*mid_vals.get(row).unwrap().get(pix).unwrap() as f64 + 1.) / 2. * ma).round() as u8;

            if *existing_luma <= sub_value {
                *values.get_mut(row).unwrap().get_mut(pix).unwrap() = 0;
            } else {
                *values.get_mut(row).unwrap().get_mut(pix).unwrap() = *existing_luma - sub_value;
            }
        }
    }

    let high_vals: Vec<Vec<f64>> = gen_heightmap(seed * 8, width, height, layer_height, hs);

    for row in 0..high_vals.len() {
        for pix in 0..high_vals[row].len() {
            let existing_luma = values.get(row).unwrap().get(pix).unwrap();
            //                                              again potential for cutoff
            let sub_value = (((255 - *existing_luma) as f64 / 255.) * (*high_vals.get(row).unwrap().get(pix).unwrap() as f64 + 1.) / 2. * ha).round() as u8;

            if *existing_luma <= sub_value {
                *values.get_mut(row).unwrap().get_mut(pix).unwrap() = 0;
            } else {
                *values.get_mut(row).unwrap().get_mut(pix).unwrap() = *existing_luma - sub_value;
            }
        }
    }

    values
}
