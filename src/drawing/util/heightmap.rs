use noise::{Billow, Fbm, NoiseFn, Perlin, PerlinSurflet, Seedable, Worley};

// returns vec of rows, with f32 between 0 <-> 1
fn gen_heightmap(seed: u32, width: usize, height: usize, base_scale: f64) -> Vec<Vec<f64>> {
    let perlin = PerlinSurflet::new(seed);

    let base_scale_coefficient = (255. / base_scale);

    let mut vec: Vec<Vec<f64>> = vec![];
    for i in 0..height {
        let mut v = vec![];
        for j in 0..width {
            v.push( perlin.get(  [ (i as f64 / width as f64) * base_scale_coefficient, (j as f64 / height as f64) * base_scale_coefficient ]  ) );
        }
        vec.push(v);
    }

    vec
}



pub fn gen_terrain(seed: u32, width: usize, height: usize, bs: f64, ba: f64, ms: f64, ma: f64, hs: f64, ha: f64) -> Vec<Vec<u8>> {
    let mut values: Vec<Vec<u8>> = Vec::new();
    for n in 0..height {
        values.push(Vec::new());
        for _ in 0..width {
            values[n].push(0);
        }
    }
    
    // let bs = 100.;
    // let ba = 200.;
    let base_vals: Vec<Vec<f64>> = gen_heightmap(seed * 2, width, height, bs);

    for row in 0..base_vals.len() {
        for pix in 0..base_vals[row].len() {
            *values.get_mut(row).unwrap().get_mut(pix).unwrap() = 255 - ((*base_vals.get(row).unwrap().get(pix).unwrap() as f64 + 1.) / 2. * ba).round() as u8;
        }
    }

    // let ms = 30.;
    // let ma = 50.;
    let mid_vals: Vec<Vec<f64>> = gen_heightmap(seed * 4, width, height, ms);

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

    // let hs = 1.;
    // let ha = 15.;
    let high_vals: Vec<Vec<f64>> = gen_heightmap(seed * 8, width, height, hs);

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
