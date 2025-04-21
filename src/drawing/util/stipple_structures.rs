use ordered_float::OrderedFloat;
use std::collections::HashMap;

///
/// Point
///
#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub struct Point {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

impl Point {
    pub fn calc_shortest_dist(&self, other: &Self) -> f32 {
        (self.x.into_inner().abs_sub(*other.x) + self.y.abs_sub(*other.y))
    }
}

///
/// Edge
///
pub struct Edge {}

impl Edge {
    // bounded intersection beteween edges (p0 -> p1) and (p2 -> p3)
    // returns some(point) if the segments intersect, with the intersection point
    pub fn bounded_intersection(p0: &Point, p1: &Point, p2: &Point, p3: &Point) -> Option<Point> {
        let denominator = *((p0.x - p1.x) * (p2.y - p3.y) - (p0.y - p1.y) * (p2.x - p3.x));

        if denominator == 0. {
            return None;
        }

        let t = ((p0.x - p2.x) * (p2.y - p3.y) - (p0.y - p2.y) * (p2.x - p3.x)).into_inner() / denominator;
        let u = ((p0.x - p2.x) * (p0.y - p1.y) - (p0.y - p2.y) * (p0.x - p1.x)).into_inner() / denominator;

        if t > 1. || t < 0. || u > 1. || u < 0. {
            return None;
        }

        Some(Point { x: OrderedFloat( p0.x.into_inner() + t * (p1.x - p0.x).into_inner() ), y: OrderedFloat( p0.y.into_inner() + t * (p1.y - p0.y).into_inner() ) })
    }
}

/// 
/// Triangle
///
pub struct Triangle {}
impl Triangle {
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

    pub fn get_edge_indexes(triangle: &[usize; 3]) -> [(usize, usize); 3] {
        [(triangle[0], triangle[1]), (triangle[1], triangle[2]), (triangle[0], triangle[2])]
    }

    // returns None if no neighbour (hull edge) else the (alredy computed) triangle index. the
    // requested triangle index is the index of the triangle trying to get its neighbour, to ignore it.
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
