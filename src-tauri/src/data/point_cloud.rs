use super::{Bounds, Point};

pub mod point_selector {
    use crate::data::Point;

    /// Select points on x axis
    pub fn x_axis(point: &Point) -> u32 {
        point.x
    }

    /// Select points on y axis
    pub fn y_axis(point: &Point) -> u32 {
        point.y
    }
}

/// A point cloud in 2D space.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PointCloud {
    points: Vec<Point>,
}

impl PointCloud {
    pub fn new<P>(points: P) -> Self
    where
        P: AsRef<[Point]>,
    {
        Self {
            points: points.as_ref().to_vec(),
        }
    }

    pub fn push(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn sort_by<Selector>(&mut self, selector: Selector)
    where
        Selector: Fn(&Point) -> u32,
    {
        self.points.sort_by_key(|point| selector(point));
    }

    fn sorted_by<Selector>(&self, selector: Selector) -> Self
    where
        Selector: Fn(&Point) -> u32,
    {
        let mut copy = self.clone();
        copy.sort_by(selector);
        copy
    }

    pub fn to_bounds(&self) -> Bounds {
        // Calculate min/max for x/y coords
        let min_x = self.points.iter().map(|point| point.x).min().unwrap_or(0);
        let min_y = self.points.iter().map(|point| point.y).min().unwrap_or(0);
        let max_x = self.points.iter().map(|point| point.x).max().unwrap_or(0);
        let max_y = self.points.iter().map(|point| point.y).max().unwrap_or(0);

        // Construct bounds from calculated values
        Bounds {
            x: min_x,
            y: min_y,
            w: max_x - min_x,
            h: max_y - min_y,
        }
    }

    pub fn cluster_by_distance<F>(&self, distance: u32, selector: F) -> Vec<PointCloud>
    where
        F: Fn(&Point) -> u32,
    {
        // Sort points by axis selector
        let points = self.sorted_by(&selector);
        let mut clusters = vec![PointCloud::default()];

        // Iterate over points and build clusters
        for point in points {
            // Fill default cluster first
            let last_cluster = clusters.last_mut().unwrap();
            if last_cluster.is_empty() {
                last_cluster.push(point);
                continue;
            }

            // Select coordinates of last pointer in active cluster and current point
            let last_coord = selector(last_cluster.points.last().unwrap());
            let curr_coord = selector(&point);

            // Decide whether to include the point in the current cluster or create a new one
            if curr_coord.abs_diff(last_coord) <= distance {
                last_cluster.push(point);
            } else {
                clusters.push(PointCloud::default());
                clusters.last_mut().unwrap().push(point);
            }
        }

        // Return clusters or empty vector if no clusters were created
        match &clusters[..] {
            [n] if n.points.is_empty() => Vec::default(),
            _ => clusters,
        }
    }
}

impl AsRef<[Point]> for PointCloud {
    fn as_ref(&self) -> &[Point] {
        &self.points
    }
}

impl Iterator for PointCloud {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.points.pop()
    }
}

impl<T> From<T> for PointCloud
where
    T: AsRef<[(u32, u32)]>,
{
    fn from(points: T) -> Self {
        Self {
            points: points
                .as_ref()
                .iter()
                .map(|point| Point::from(*point))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{point_selector, Bounds, Point, PointCloud};

    #[test]
    fn test_cluster_by_distance() {
        let distance = 5;
        let points: Vec<Point> = [(0, 0), (9, 1), (5, 1), (15, 5), (17, 3)]
            .into_iter()
            .map(|x| x.into())
            .collect();
        let cloud = PointCloud::new(points);
        let clusters = cloud.cluster_by_distance(distance, point_selector::x_axis);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0], PointCloud::from([(0, 0), (5, 1), (9, 1)]));
        assert_eq!(clusters[1], PointCloud::from([(15, 5), (17, 3)]));
    }

    #[test]
    fn test_approx_rect() {
        let bounds = PointCloud::from([(0, 0), (10, 10)]).to_bounds();
        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 0);
        assert_eq!(bounds.w, 10);
        assert_eq!(bounds.h, 10);

        let bounds = PointCloud::from([(5, 3), (7, 9), (10, 5)]).to_bounds();
        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 3);
        assert_eq!(bounds.w, 5);
        assert_eq!(bounds.h, 6);
    }

    #[test]
    fn test_bounds() {
        let bounds = Bounds {
            x: 50,
            y: 50,
            w: 50,
            h: 50,
        };
        let bounds = bounds.grow_by(10);
        assert_eq!(bounds.x, 45);
        assert_eq!(bounds.y, 45);
        assert_eq!(bounds.w, 60);
        assert_eq!(bounds.h, 60);

        let bounds = Bounds {
            x: 50,
            y: 50,
            w: 50,
            h: 50,
        };
        let center = bounds.center();
        assert_eq!(center, (75, 75).into());

        let bounds = Bounds {
            x: 50,
            y: 50,
            w: 50,
            h: 50,
        };
        assert!(!bounds.contains_point(&Point::new(49, 49)));
        assert!(!bounds.contains_point(&Point::new(101, 101)));
        assert!(bounds.contains_point(&Point::new(50, 50)));
        assert!(bounds.contains_point(&Point::new(100, 100)));
    }
}
