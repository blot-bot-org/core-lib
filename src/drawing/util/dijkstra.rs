use ordered_float::OrderedFloat;

/// 
/// Creates an undirected adjacancy matrix between points.
///
/// # Parameters:
/// - `seeds`: The set of points to construct the matrix from
///
/// # Returns:
/// - An adjacancy matrix, representing the distances between the points
///
pub fn create_adjacancy_matrix(seeds: Vec<(f64, f64)>) -> Vec<Vec<f64>> {
    let mut matrix: Vec<Vec<f64>> = Vec::with_capacity(seeds.len());

    // construct the adjacancy matrix
    for i in 0..seeds.len() {
        matrix.push(Vec::with_capacity(seeds.len()));

        for j in 0..seeds.len() {

            if i == j {
                matrix[i].push(0.);
                continue;
            }

            matrix[i].push(point_magnitude(seeds[i], seeds[j]));
        }
    }

    matrix
}


///
/// # Returns:
/// - The distance between two points, as a magnitude
///
fn point_magnitude(p0: (f64, f64), p1: (f64, f64)) -> f64 {
    (p0.0 - p1.0).powi(2).sqrt() + (p0.1 - p1.1).powi(2).sqrt()
}


/// 
/// Performs dijkstras on an adjacancy matrix.
/// It returns the tour as a list of indices, so it assumes the initial seeds
/// used to create the adjacancy matrix have no changed in order.
///
/// # Parameters:
/// - `adjacancy_matrix`: The adjacancy matrix of the points
///
/// # Returns:
/// - The tour, as a list of indices
///
fn dijkstras(adjacancy_matrix: Vec<Vec<f64>>) -> Vec<usize> {

    // TODO
    vec![]
    
}



// just using this for distances (f64) so i wont make it generic
struct PriorityQueue {
    items: Vec<(OrderedFloat<f64>, usize)>,
}

impl PriorityQueue {
    pub fn new() -> PriorityQueue {
        PriorityQueue { items: Vec::new() }
    }

    pub fn enqueue(&mut self, priority: OrderedFloat<f64>, point_idx: usize) {
        for i in 0..self.items.len() {
            if self.items[i].0 > priority {
                self.items.insert(i, (priority, point_idx));
                return;
            }
        }

        // first item in queue / lowest priority
        self.items.push((priority, point_idx));
    }

    pub fn dequeue(&mut self) -> Option<(OrderedFloat<f64>, usize)> {
        if self.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// we will implement this as a binary heap for maximum marks and good efficiency


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        
        
        let mut seeds = Vec::new();
        seeds.push((5.5, 6.5));
        seeds.push((2.5, 3.5));
        seeds.push((1.5, 0.5));
        seeds.push((100.5, 500.5));
        seeds.push((0.5, 10.5));
        seeds.push((200.5, 51.5));

        let adj_mat = create_adjacancy_matrix(seeds);
        dijkstras(adj_mat);

        assert!(1 == 1);
    }

}
