use ordered_float::OrderedFloat;
use std::collections::HashMap;


///
/// A representation of a 2D point.
///
/// # Fields:
/// - `x`: An f32 for the x component
/// - `y`: An f32 for the y component
///
#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub struct Point {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

impl Point {
    ///
    /// Calculates the distance to the given neighbour point.
    /// 
    /// # Returns:
    /// - An f32 representing the direct distance
    ///
    pub fn calc_shortest_dist(&self, other: &Self) -> f32 {
        (self.x.into_inner() - *other.x).max(0.) + (self.y.into_inner() - *other.y).max(0.)
    }
}

///
/// An empty struct, with an implemented edge-related function.
///
pub struct Edge {}

impl Edge {
    ///
    /// Checks whether an intersection occurs between two finite edges. The parameters are
    /// the endpoints of the edges.
    ///
    /// # Parameters:
    /// - `p0`: An endpoint of the first edge
    /// - `p1`: An endpoint of the first edge
    /// - `p2`: An endpoint of the second edge
    /// - `p3`: An endpoint of the second edge
    ///
    /// # Returns:
    /// - `None` if no intersection occurs
    /// - `Some(Point)` if an intersection does occur, returning the point of intersection
    ///
    pub fn bounded_intersection(p0: &Point, p1: &Point, p2: &Point, p3: &Point) -> Option<Point> {
        let denominator = *((p0.x - p1.x) * (p2.y - p3.y) - (p0.y - p1.y) * (p2.x - p3.x));

        if denominator == 0. {
            return None;
        }

        let t = ((p0.x - p2.x) * (p2.y - p3.y) - (p0.y - p2.y) * (p2.x - p3.x)).into_inner() / denominator;
        let u = ((p0.x - p2.x) * (p0.y - p1.y) - (p0.y - p2.y) * (p0.x - p1.x)).into_inner() / denominator;

        // check t and u coefficients, if they're between 0 and 1 an intersection occured
        if t > 1. || t < 0. || u > 1. || u < 0. {
            return None;
        }

        Some(Point { x: OrderedFloat( p0.x.into_inner() + t * (p1.x - p0.x).into_inner() ), y: OrderedFloat( p0.y.into_inner() + t * (p1.y - p0.y).into_inner() ) })
    }
}

/// 
/// An empty struct, with implemented triangle-related functions.
///
pub struct Triangle {}

impl Triangle {
    /// 
    /// Calculates the circumcenter of the triangle, using the localised cartesian coordinates
    /// equations.
    /// Reference equations: https://en.wikipedia.org/wiki/Circumcircle#Cartesian_coordinates_2
    ///
    /// # Parameters:
    /// - `p0`: A vertex of the triangle
    /// - `p1`: A vertex of the triangle
    /// - `p2`: A vertex of the triangle
    ///
    /// # Returns:
    /// - The circumcenter of the triangle, as a `Point`
    ///
    pub fn circumcenter(p0: &Point, p1: &Point, p2: &Point) -> Point {
        let (p1x, p1y) = (p1.x - p0.x, p1.y - p0.y);
        let (p2x, p2y) = (p2.x - p0.x, p2.y - p0.y);
        let mut prime_d = OrderedFloat(2. * (p1x.into_inner() * p2y.into_inner() - p1y.into_inner() * p2x.into_inner()));

        if prime_d == 0. {
            prime_d = OrderedFloat(1.);
        }
        
        let upx = (1. / prime_d.into_inner()) * (
            p2y.into_inner() * (p1x.powi(2) + p1y.powi(2)) - p1y.into_inner() * (p2x.powi(2) + p2y.powi(2))
        );
        let upy = (1. / prime_d.into_inner()) * (
            p1x.into_inner() * (p2x.powi(2) + p2y.powi(2)) - p2x.into_inner() * (p1x.powi(2) + p1y.powi(2))
        );

        let ux = OrderedFloat(upx) + p0.x;
        let uy = OrderedFloat(upy) + p0.y;

        Point { x: ux, y: uy }
    }

    ///
    /// Checks whether a given point is within the circumcircle of a triangle.
    /// The circumcircle is calculated using the localised cartesian coordinates equations.
    /// Reference equations: https://en.wikipedia.org/wiki/Circumcircle#Cartesian_coordinates_2
    ///
    /// # Parameters:
    /// - `test_point`: The point to be tested, whether in or out of circumcircle
    /// - `p0`: A vertex of the triangle
    /// - `p1`: A vertex of the triangle
    /// - `p2`: A vertex of the triangle
    ///
    /// # Returns:
    /// - A boolean, true if the point was in the circumcircle
    pub fn point_in_circle(test_point: &Point, p0: &Point, p1: &Point, p2: &Point) -> bool {
        let (p1x, p1y) = (p1.x - p0.x, p1.y - p0.y);
        let (p2x, p2y) = (p2.x - p0.x, p2.y - p0.y);
        let mut prime_d = OrderedFloat(2. * (p1x.into_inner() * p2y.into_inner() - p1y.into_inner() * p2x.into_inner()));

        if prime_d == 0. {
            prime_d = OrderedFloat(1.);
        }
        
        let upx = (1. / prime_d.into_inner()) * (
            p2y.into_inner() * (p1x.powi(2) + p1y.powi(2)) - p1y.into_inner() * (p2x.powi(2) + p2y.powi(2))
        );
        let upy = (1. / prime_d.into_inner()) * (
            p1x.into_inner() * (p2x.powi(2) + p2y.powi(2)) - p2x.into_inner() * (p1x.powi(2) + p1y.powi(2))
        );

        let ux = OrderedFloat(upx) + p0.x;
        let uy = OrderedFloat(upy) + p0.y;
        let radius = (upx.powi(2) + upy.powi(2)).sqrt();

        let dist = ((ux - test_point.x).powi(2) + (uy - test_point.y).powi(2)).sqrt();

        dist <= radius
    }

    /// 
    /// # Returns:
    /// - The 3 edges of a given set of 3 points
    ///
    pub fn get_edge_indexes(triangle: &[usize; 3]) -> [(usize, usize); 3] {
        [(triangle[0], triangle[1]), (triangle[1], triangle[2]), (triangle[0], triangle[2])]
    }

    /// 
    /// Helper function to calculate a neighbouring triangle, given an edge.
    /// It uses a pre-calculated map, where edge indices -> a set of two triangles, which share the
    /// edge.
    ///
    /// # Parameters:
    /// - `requested_triangle_index`: The index of the requesting triangle, so it can ignore itself
    /// as a neighbour
    /// - `edge_indices`: The indices of the points that form the edge
    /// - `edge_triangle`: The map containing edges and their related triangle(s)
    ///
    /// # Returns:
    /// - `None` if no triangle shares the edge (in the context of a delaunay triangulation, this
    /// means the triangle is on the convex hull)
    /// - `Some(usize)` if there is a neighbouring triangle, with the index of the neighbouring
    /// triangle
    ///
    pub fn get_neighbouring_triangle(requested_triangle_index: usize, edge_indices: (usize, usize), edge_triangle: &HashMap<(usize, usize), (usize, usize)>) -> Option<usize> {
        let key = if edge_indices.0 > edge_indices.1 { (edge_indices.0, edge_indices.1) } else { (edge_indices.1, edge_indices.0) }; 

        if let Some((i0, i1)) = edge_triangle.get(&key) {
            if *i1 == usize::MAX { // only one triangle, its the requested one
                return None;
            }

            if *i0 == requested_triangle_index {
                return Some(*i1);
            } else if *i1 == requested_triangle_index {
                return Some(*i0);
            } else {
                panic!("The triangle asked for it's edge neighbours, yet it didn't have that edge");
            }
        }

        None
    }
}
