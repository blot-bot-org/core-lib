use crate::drawing::util::stipple_structures::*;
use image::{ImageBuffer, ImageReader};
use rand::Rng;
use ordered_float::OrderedFloat;
use std::collections::HashMap;


/// 
/// Main function to stipple points. This function plots the initial points,
/// weighted towards darker areas of the image, and then calls n iterations of Lloyd's relaxation.
/// 
///
/// # Parameters:
/// - `file_path`: The path of the input image file
/// - `num_points`: The number of points to stipple
/// - `iterations`: The number of iterations of Lloyd's relaxation to perform
/// - `relaxation_tendency`: The coefficient for Lloyd's relaxation
/// - `brightness_threshold`: The luma value which below pixels are seeded
///
/// # Returns
/// - A vector containing the positions of the stippled points
/// - An error explaining why the stipple failed
///
pub fn stipple_points(file_path: &str, num_points: usize, iterations: usize, relaxation_tendency: f32, brightness_threshold: u8) -> Result<Vec<Point>, String> {

    // open input image
    let input_image = match ImageReader::open(file_path) {
        Ok(img) => {
            img.decode().unwrap().into_rgb8()
        },
        Err(err) => {
            return Err(format!("Error loading image. {}", err.to_string()).to_owned());
        }
    };
    
    // create list of points, place them randomly at darker areas of image
    let mut points: Vec<Point> = Vec::with_capacity(num_points);
    let mut points_placed = 0;
    let mut rng = rand::rng();

    while points_placed < num_points {
        let rand_x = rng.random::<f32>() * input_image.width() as f32;
        let rand_y = rng.random::<f32>() * input_image.height() as f32;

        let pixel = input_image.get_pixel(rand_x as u32, rand_y as u32).0;
        if (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3 < ((rng.random::<f32>() * brightness_threshold as f32 * rng.random::<f32>()) as u32) {
            points.push(Point { x: OrderedFloat(rand_x), y: OrderedFloat(rand_y) });
            points_placed += 1;
        }
    }

    // iterate the lloyd's relaxation n times
    for _ in 0..iterations {
        if let Err(err_str) = iterate(&mut points, &input_image, relaxation_tendency) {
            return Err(err_str);
        };
    }

    Ok(points)
}


///
/// Performs an iteration of relaxation on a list of points, changing the points in place.
/// The function calls a delaunay triangulation, creates the voronoi diagram, and then implements
/// the Lloyd's relaxation algorithm.
///
/// # Parameters:
/// - `points`: A mutable list of input points
/// - `input_image`: The loaded input image
/// - `relaxation_tendency`: A scalar float representing the tendency / strength of the cell relaxation
///
/// # Returns:
/// - Void if an iteration suceeded
/// - An error as an owned string, explaining why the function failed
///
fn iterate(points: &mut Vec<Point>, input_image: &ImageBuffer<image::Rgb<u8>, Vec<u8>>, relaxation_tendency: f32) -> Result<(), String> {
    
    // computes the delaunay triangulation
    let (triangles, new_points) = match bowyer_watson(points) {
        Ok((tri, n_p)) => (tri, n_p),
        Err(err_str) => return Err(err_str),
    };

    let edge_triangles: HashMap<(usize, usize), (usize, usize)> = match get_edge_triangles(&triangles) {
        Ok(val) => val,
        Err(err_str) => return Err(err_str),
    };
    
    // computes the voronoi diagram
    let (voronoi_sites, _voronoi_edges, site_vertices) = match get_extended_voronoi(&new_points, &triangles, &edge_triangles, (input_image.width() as f32, input_image.height() as f32)) {
        Ok((vs, ve, sv)) => (vs, ve, sv),
        Err(err_str) => return Err(err_str),
    };

    // performs the weghted lloyd's stippling, tending cell sites towards the cell centroids given
    // a scalar `relaxation_tendency`
    for (index, (&site, neighbours)) in site_vertices.iter().enumerate() {
        let mut sum_weighted_x = 0.;
        let mut sum_weighted_y = 0.;
        let mut total_weight = 0.;

        for n in neighbours.iter() {
            let image_x = ((voronoi_sites[*n].x).into_inner() as u32).min(input_image.width() - 1).max(0);
            let image_y = ((voronoi_sites[*n].y).into_inner() as u32).min(input_image.height() - 1).max(0);

            let pixel = input_image.get_pixel(image_x, image_y);
            let weight = (255. - ((pixel.0[0] as f32 + pixel.0[1] as f32 + pixel.0[2] as f32) / 3.)) / 255.;

            sum_weighted_x += *voronoi_sites[*n].x.min(OrderedFloat(input_image.width() as f32)).max(OrderedFloat(0.)) * weight;
            sum_weighted_y += *voronoi_sites[*n].y.min(OrderedFloat(input_image.height() as f32)).max(OrderedFloat(0.)) * weight;
            total_weight += weight;
        }

        let centroid_x = sum_weighted_x / total_weight.max(1.);
        let centroid_y = sum_weighted_y / total_weight.max(1.);

        let lerp_x = new_points[site].x + (centroid_x - *new_points[site].x) * relaxation_tendency;
        let lerp_y = new_points[site].y + (centroid_y - *new_points[site].y) * relaxation_tendency;

        points[index] = Point { x: lerp_x, y: lerp_y };
    }

    Ok(())
}


///
/// Performs the nearest neighbour pathfinding algorithm on a given set of points.
/// I use nearest neighbour only to create a path for the pen to follow - hence a bad,
/// heuristic pathfinding algorithm is not the end of the world.
///
/// # Parameters:
/// - `points`: A list of points to perform the pathfinding algorithm on
///
/// # Returns:
/// - A new vector, the tour, representing the indices of the points in order
///
pub fn nearest_neighbour_tour(points: &Vec<Point>) -> Vec<usize> {
    let mut visited = vec![false; points.len()];
    
    let mut tour: Vec<usize> = Vec::with_capacity(points.len());
    let mut current_idx = 0;

    tour.push(current_idx);
    visited[current_idx] = true;

    // repeatedly find the next closest point, and add it to the tour
    for _ in 1..points.len() {
        let mut nearest = None;
        let mut nearest_distance = f32::INFINITY;

        for k in 0..points.len() {
            if !visited[k] {
                let dist = points[current_idx].calc_shortest_dist(&points[k]);
                if dist < nearest_distance {
                    nearest_distance = dist;
                    nearest = Some(k);
                }
            }
        }

        if let Some(next_idx) = nearest {
            current_idx = next_idx;
            tour.push(current_idx);
            visited[current_idx] = true;
        }
    }

    tour
}


/// 
/// Computes the delaunay triangulation, given a set of points.
/// This function is an implementation of the Bowyer-Watson algorithm.
/// Pseudocode reference: https://en.wikipedia.org/wiki/Bowyer%E2%80%93Watson_algorithm#Pseudocode
///
/// # Parameters:
/// - `points`: The list of points of which to compute the delaunay triangulation
///
/// # Returns:
/// - A new vector of arrays, where each array of 3 indices points to the 3 vertices of a triangle
/// - A list of points with the super-triangle vertices
///
fn bowyer_watson(points: &Vec<Point>) -> Result<(Vec<[usize; 3]>, Vec<Point>), String> {

    // single copy to vec occurs here
    let mut all_points = points.to_vec();

    let super_triangle = get_super_triangle(points);
    let super_triangle_index = all_points.len();
    all_points.push(super_triangle[0].clone());
    all_points.push(super_triangle[1].clone());
    all_points.push(super_triangle[2].clone());

    // 3 consecutive integers are indices of all_points, which form a triangle
    let mut triangle_indices: Vec<[usize; 3]> = vec![[super_triangle_index, super_triangle_index + 1, super_triangle_index + 2]];

    // doesn't iterate super_triangle points
    for point_idx in 0..points.len() {

        let mut bad_triangles: Vec<usize> = vec![]; // indices of arrays in triangle_indices

        // the index of the index set in `triangle_indicies`
        for index_set_index in 0..triangle_indices.len() {
            if Triangle::point_in_circle(&all_points[point_idx], all_points.get(triangle_indices[index_set_index][0]).unwrap(), all_points.get(triangle_indices[index_set_index][1]).unwrap(), all_points.get(triangle_indices[index_set_index][2]).unwrap()) {
                bad_triangles.push(index_set_index);
            }
        }

        let mut bad_edges: Vec<(usize, usize)> = vec![];
        // add the edge tuples to the vector, whilst normalising to make edge a <-> b == b <-> a
        for indice_index in bad_triangles.iter() {
            let val = &triangle_indices[*indice_index];

            if val[0] > val[1] {
                bad_edges.push((val[0], val[1]));
            } else {
                bad_edges.push((val[1], val[0]));
            }

            if val[1] > val[2] {
                bad_edges.push((val[1], val[2]));
            } else {
                bad_edges.push((val[2], val[1]));
            }

            if val[2] > val[0] {
                bad_edges.push((val[2], val[0]));
            } else {
                bad_edges.push((val[0], val[2]));
            }
        }

        // nb: flamegraph tests show hashmap allocation is using moderate execution expense
        let mut edge_count = HashMap::new();
        for &(a, b) in bad_edges.iter() {
            *edge_count.entry((a, b)).or_insert(0) += 1;
        }

        let mut polygon: Vec<(usize, usize)> = vec![];
        for edge in bad_edges.iter() {
            if let Some(ec) = edge_count.get(edge) {
                if *ec == 1 {
                    polygon.push(*edge);
                }
            } else {
                return Err("All delaunay edges should have HashMap entry.".to_owned());
            }
        }

        for bad_triangle_index in bad_triangles.iter().rev() { // reverse iterator to preverse index ordering
            triangle_indices.remove(*bad_triangle_index);
        }

        for &(a, b) in polygon.iter() {
            let mut new_tri = [a, b, point_idx];
            new_tri.sort();
            triangle_indices.push(new_tri);
        }
    }

    // remove all triangles connected to super_triangle
    triangle_indices.retain(|tri| !(tri.contains(&super_triangle_index) || tri.contains(&(super_triangle_index + 1)) || tri.contains(&(super_triangle_index + 2))));

    Ok((triangle_indices, all_points))
}


/// 
/// Computes the triangles which are part of a specific edge.
///
/// A map is created, where the keys are normalised tuples of the edge, and the values are either
/// one or two triangles which share that edge.
/// If an edge only has one triangle, it's value will look like (triangle_indice, usize::MAX).
/// This means the triangle is on the convex hull of the delaunay triangulation.
///
/// # Parameters:
/// - `triangles`: A vector of arrays of triangle indices
///
/// # Returns:
/// - A HashMap of edges <-> triangles as described above
/// - An error as an owned string, explaining the error
///
fn get_edge_triangles(triangles: &Vec<[usize; 3]>) -> Result<HashMap<(usize, usize), (usize, usize)>, String> {
    // theoretically, if there are 18446744073709551615 or more points, we have a problem.
    if triangles.len() >= usize::MAX {
        return Err("There were too many triangles to safely set null to usize::MAX".to_owned());
    }

    // normalised edge (usize usize) <-> (usize, usize) pointers to triangles
    // by default, the pointers to triangles are usize::MAX. each tuple will have either 2 or 1
    // indexes, if it has 1 index and one usize::MAX, it is a hull edge.
    let mut edge_triangle: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    for (index, triangle) in triangles.iter().enumerate() {
        for (edge_idx0, edge_idx1) in Triangle::get_edge_indexes(&triangle) {
            let key = if edge_idx0 > edge_idx1 { (edge_idx0, edge_idx1) } else { (edge_idx1, edge_idx0) };

            edge_triangle.entry(key)
                .and_modify(|value| {
                    if value.1 == usize::MAX { value.1 = index } else { /* println!("Edge already has two references"); */ value.1 = index; }
                })
                .or_insert_with(|| ((index, usize::MAX)));
        }
    }

    Ok(edge_triangle)
}


/// 
/// Computes the voronoi diagram. Specifically, this function:
///     1. Computes a partial voronoi diagram.
///     2. Extends edge rays, away from the centroid of the delaunay triangulation.
///     3. Clips the voronoi cells to a bounding box, to complete the voronoi diagram.
///     4. Amidst this, lazily computes the corresponding voronoi sites to their voronoi cells.
/// An initial Google search told me I just had to do step 1. Well constructing a voronoi diagram isn't as easy
/// as Google makes it seem. Worse, Google has basically no information on steps 2, 3 and 4, so I
/// had to figure it out for myself. I hated every second of writing this function.
///
/// # Parameters:
/// - `points`: A list of points which form the vertices of the delaunay triangulation
/// - `triangles`: A list of triangle arrays, which are 3 indices representing point indices
/// - `edge_triangles`: A HashMap to lookup which edge makes which triangle(s)
/// - `max_wh`: The width/height to bound the diagram to
///
/// # Returns:
/// - A vector of the voronoi diagram's indices
/// - A vector containing two indices of the point vector, which form the edges of the voronoi diagram
/// - A HashMap of a site, with a key as an index of the point vector, and a list of
///   corresponding indices of the point vector, which form the polygon of the voronoi cell
/// - An error as an owned string, explaining the error
///
fn get_extended_voronoi(points: &Vec<Point>, triangles: &Vec<[usize; 3]>, edge_triangles: &HashMap<(usize, usize), (usize, usize)>, max_wh: (f32, f32)) -> Result<(Vec<Point>, Vec<(usize, usize)>, HashMap<usize, Vec<usize>>), String> {
    // vector, the index of the site point corresponds to the index of the triangle in `triangles`
    let mut voronoi_sites: Vec<Point> = Vec::with_capacity(triangles.len());
    voronoi_sites.extend(std::iter::repeat(Point { x: OrderedFloat(0.), y: OrderedFloat(0.) }).take(triangles.len()));

    // this array has a tuple of pointers to the voronoi sites (to form an edge)
    // KEEP THESE TUPLES NORMALISED PLEASE!
    let mut voronoi_edges: Vec<(usize, usize)> = vec![];

    // here we go through every delaunay triangle, and calculate its circumcenter for a voronoi vertex
    for (index, triangle) in triangles.iter().enumerate() {
        if let Some(value) = voronoi_sites.get_mut(index) {
            *value = Triangle::circumcenter(&points[triangle[0]], &points[triangle[1]], &points[triangle[2]]);
        } else {
            return Err("There should've been as many site points as there are triangles".to_owned());
        }
    }

    // lazily compute hull edges
    let mut hull_point_tri: Vec<((usize, usize), usize)> = vec![];
    let mut site_vertices: HashMap<usize, Vec<usize>> = HashMap::new();
    
    // for every edge, and its corresponding triangles
    for ((p0, p1), (t0, t1)) in edge_triangles.iter() {

        if *t1 == usize::MAX {
            // just building this up whilst iterating through, used in the hull extension later
            hull_point_tri.push(((*p0, *p1), *t0));
            continue;
        }
        
        // we can use the triangle index in this scenario, as `voronoi_sites` uses the triangle
        // index to correspond to the site index
        //
        // im also pushing the two site points, and the four respective voronoi vertices
        if *t0 > *t1 {
            voronoi_edges.push((*t0, *t1));
        } else {
            voronoi_edges.push((*t1, *t0));
        }
        
        // the t0 and t1 correspond to the voronoi_sites indices
        site_vertices.entry(*p0).or_insert(Vec::new());
        if !site_vertices.get_mut(p0).unwrap().contains(t0) {
            site_vertices.get_mut(p0).unwrap().push(*t0);
        }
        if !site_vertices.get_mut(p0).unwrap().contains(t1) {
            site_vertices.get_mut(p0).unwrap().push(*t1);
        }
        site_vertices.entry(*p1).or_insert(Vec::new());
        if !site_vertices.get_mut(p1).unwrap().contains(t0) {
            site_vertices.get_mut(p1).unwrap().push(*t0);
        }
        if !site_vertices.get_mut(p1).unwrap().contains(t1) {
            site_vertices.get_mut(p1).unwrap().push(*t1);
        }
    }

    // sort the voronoi vertices around each site point by angle
    for (site, neighbours) in site_vertices.iter_mut() {
        neighbours.sort_by(|n0, n1| {
            let angle_n0 = (voronoi_sites[*n0].y - points[*site].y).atan2(*(voronoi_sites[*n0].x - points[*site].x));
            let angle_n1 = (voronoi_sites[*n1].y - points[*site].y).atan2(*(voronoi_sites[*n1].x - points[*site].x));

            angle_n0.partial_cmp(&angle_n1).unwrap()
        });

        
    }


    // should be normalised but no harm in meaning each point twice, to be sure
    let hull_centroid = Point {
        x: hull_point_tri.iter().map(|((p0, p1), _)| points[*p0].x + points[*p1].x).sum::<OrderedFloat<f32>>() / (OrderedFloat((hull_point_tri.len() * 2) as f32)),
        y: hull_point_tri.iter().map(|((p0, p1), _)| points[*p0].y + points[*p1].y).sum::<OrderedFloat<f32>>() / (OrderedFloat((hull_point_tri.len() * 2) as f32))
    };

    // the strategy I have devised for the hull extension is:
    // 1. consider just the hull of the delaunay triangulation (in hull_point_tri)
    // 2. cast a ray, from the circumcenter of the edges triangle, in both directions
    // 3. then there are two scenarios:
    //    - each cast had 1 intersection (find the absolute distance between the two -> shortest distance is direction to go)
    //    - one cast had 0 intersections, one cast had 2+ (direction should be 0 intersections direction)
    for ((p0, p1), t0) in hull_point_tri.iter() {

        
        let mid_x = *(points[*p0].x + points[*p1].x) / 2.;
        let mid_y = *(points[*p0].y + points[*p1].y) / 2.;

        let vector = (mid_x - *voronoi_sites[*t0].x, mid_y - *voronoi_sites[*t0].y);
        let normalisation_denominator = (vector.0.powi(2) + vector.1.powi(2)).sqrt();

        let normalised_vector = (vector.0 / normalisation_denominator, vector.1 / normalisation_denominator);
        // DIMENSION REF!
        let mut scalar = ((max_wh.0.max(max_wh.1)).powi(2)).sqrt() * 2.; // 10 * dimension

        let positive_dot = (normalised_vector.0 * (voronoi_sites[*t0].x - hull_centroid.x).into_inner()) + (normalised_vector.1 * (voronoi_sites[*t0].y - hull_centroid.y).into_inner());
        if positive_dot < 0. { // pointing towards the mesh
            scalar *= -1.;
        }


        let perp_p0 = voronoi_sites[*t0];
        let perp_p1 = Point { x: OrderedFloat(*voronoi_sites[*t0].x + normalised_vector.0 * scalar), y: OrderedFloat(*voronoi_sites[*t0].y + normalised_vector.1 * scalar) };

        let idx = voronoi_sites.len();
        voronoi_edges.push((idx, idx+1));
        voronoi_sites.push(perp_p0);
        voronoi_sites.push(perp_p1);

        // the t0 and t1 correspond to the voronoi_sites indices
        site_vertices.entry(*p0).or_insert(Vec::new());
        if !site_vertices.get_mut(p0).unwrap().contains(&(idx + 1)) {
            site_vertices.get_mut(p0).unwrap().push(idx + 1);
        }
        site_vertices.entry(*p1).or_insert(Vec::new());
        if !site_vertices.get_mut(p1).unwrap().contains(&(idx + 1)) {
            site_vertices.get_mut(p1).unwrap().push(idx + 1);
        }

    }


    // finally trim the points to a bounding box, repeat for t/r/b/;
    // 1. create bounding edge, as well as point at (0, 0) for top, (1000, 0) for right etc
    // 2. find interesections with edges, store edge index with point
    // 3. order by point (depending on t/r/b/l)
    // 4. for all intersections:
    //     - create point at intersection.
    //     - modify edge to have the extreme point index (outside bounds) to be newly created point index.
    //     - loop through sites, find references to old point, update them to new point
    //     - remove the "site point" which has just been replaced, from the point list (cant do that)
    //     - join previous intersection point and current point
    //     - set previous point to current point
    // then join the previous point with (0, 0) point
   
  
    // stores the edge index -> the trimmed point
    let mut intersection_points: Vec<(usize, Point)> = vec![];
    let mut dead_site_points: Vec<usize> = vec![];
    let bounds = [
        Point { x: OrderedFloat(0.), y: OrderedFloat(0.) },
        Point { x: OrderedFloat(max_wh.0), y: OrderedFloat(0.) },
        Point { x: OrderedFloat(max_wh.0), y: OrderedFloat(max_wh.1) },
        Point { x: OrderedFloat(0.), y: OrderedFloat(max_wh.1) }
    ];

    // first we calculate the intersections
    for i in 0..4 {
        let mut local_intersection_points: Vec<(usize, Point)> = vec![];

        let bound_p0 = &bounds[i];
        let bound_p1 = &bounds[(i + 1) % 4];
        for (index, edge) in voronoi_edges.iter().enumerate() {
            if let Some(point) = Edge::bounded_intersection(bound_p0, bound_p1, &voronoi_sites[edge.0], &voronoi_sites[edge.1]) {
                local_intersection_points.push((index, point));
            };
        }

        local_intersection_points.sort_by_key(|o| if i % 2 == 0 { o.1.x } else { o.1.y });
        if bound_p0.y == max_wh.1 {
            local_intersection_points.reverse();
        }

        intersection_points.extend(local_intersection_points);
    }
    
    let mut last_point_idx: Option<usize> = None;
    let mut first_index = 0_usize; // used for the final join, to cycle it

    // now we go through the intersections and connect the points
    for intersection_idx in 0..intersection_points.len() {
        let (edge_index, point) = intersection_points[intersection_idx];

        let new_site_point_idx = voronoi_sites.len();
        voronoi_sites.push(point);

        // now we can quickly update the voronoi sites to have the correct vertex pointers
        // -> `voronoi_edges[index].1` contains the pointer to the illegal vertex
        // so loop through each site, if any reference to old vertices, update it
        for (_site_index, vertices) in site_vertices.iter_mut() {
            if voronoi_sites[voronoi_edges[edge_index].0].x.into_inner() > max_wh.0 || voronoi_sites[voronoi_edges[edge_index].0].x.into_inner() < 0. || voronoi_sites[voronoi_edges[edge_index].0].y.into_inner() > max_wh.1 || voronoi_sites[voronoi_edges[edge_index].0].y.into_inner() < 0. {
                if let Some(idx) = vertices.iter().position(|&p0| p0 == voronoi_edges[edge_index].0) {
                    let _ = vertices.remove(idx);
                    vertices.push(new_site_point_idx);
                }
            }
            if voronoi_sites[voronoi_edges[edge_index].1].x.into_inner() > max_wh.0 || voronoi_sites[voronoi_edges[edge_index].1].x.into_inner() < 0. || voronoi_sites[voronoi_edges[edge_index].1].y.into_inner() > max_wh.1 || voronoi_sites[voronoi_edges[edge_index].1].y.into_inner() < 0. {
                if let Some(idx) = vertices.iter().position(|&p0| p0 == voronoi_edges[edge_index].1) {
                    let _ = vertices.remove(idx);
                    vertices.push(new_site_point_idx);
                }
            }
        }


        let first_point = voronoi_sites[voronoi_edges[edge_index].0]; // get the first point of the edge
        // DIMENSION REF!
        if first_point.x.into_inner() > max_wh.0 || first_point.x.into_inner() < 0. || first_point.y.into_inner() > max_wh.1 || first_point.y.into_inner() < 0. {
            dead_site_points.push(voronoi_edges[edge_index].0);
            voronoi_edges[edge_index] = (new_site_point_idx, voronoi_edges[edge_index].1);
        } else {
            dead_site_points.push(voronoi_edges[edge_index].1);
            voronoi_edges[edge_index] = (voronoi_edges[edge_index].0, new_site_point_idx);
        }

        if let Some(last_idx) = last_point_idx {
            voronoi_edges.push((last_idx, new_site_point_idx));
        } else {
            first_index = new_site_point_idx;
        }
        last_point_idx = Some(new_site_point_idx);
    }
    if let Some(lpidx) = last_point_idx {
        voronoi_edges.push((lpidx, first_index));
    } else {
        return Err("There was no last_point_idx when bounding voronoi diagram. Was a diagram created?".to_owned());
    }
    
    Ok((voronoi_sites, voronoi_edges, site_vertices))
}


/// 
/// Computes the size of the initial super triangle for the delaunay triangulation.
/// The super triangle must enclose all given points.
///
/// # Parameters:
/// - `points`: The points of which to create the super triangle on
///
/// # Returns:
/// - An array of 3 points which form the super triangle
///
fn get_super_triangle(points: &[Point]) -> [Point; 3] {
    let max_x = points.iter().max_by_key(|p| p.x).unwrap().x * 2.;
    let max_y = points.iter().max_by_key(|p| p.y).unwrap().y * 2.;

    [ Point { x: OrderedFloat(0.), y: OrderedFloat(0.) }, Point { x: max_x, y: OrderedFloat(0.) }, Point { x: OrderedFloat(0.), y: max_y } ]
}
