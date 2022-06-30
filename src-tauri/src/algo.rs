use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ApproxRect {
    points: Vec<(u32, u32)>,
}

impl ApproxRect {
    pub fn new(points: Vec<(u32, u32)>) -> Self {
        Self { points }
    }

    pub fn into_bounds(self) -> Bounds {
        let min_x = self.points.iter().map(|(x, _)| *x).min().unwrap();
        let min_y = self.points.iter().map(|(_, y)| *y).min().unwrap();
        let max_x = self.points.iter().map(|(x, _)| *x).max().unwrap();
        let max_y = self.points.iter().map(|(_, y)| *y).max().unwrap();
        Bounds {
            x: min_x,
            y: min_y,
            w: max_x - min_x,
            h: max_y - min_y,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Bounds {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Bounds {
    pub fn get_lowest_center_point(&self) -> (u32, u32) {
        (self.x + self.w / 2, self.y + self.h)
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.w as usize * self.h as usize
    }

    pub fn grow_by(&self, px: u32) -> Bounds {
        Bounds {
            x: self.x.saturating_sub(px / 2),
            y: self.y.saturating_sub(px / 2),
            w: self.w + px,
            h: self.h + px,
        }
    }

    #[inline]
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    #[inline]
    pub fn contains_point(&self, point: &(u32, u32)) -> bool {
        point.0 >= self.x
            && point.0 < self.x + self.w
            && point.1 >= self.y
            && point.1 < self.y + self.h
    }
}

#[derive(Debug, Default)]
pub struct AxisCluster {
    points: Vec<(u32, u32)>,
}

impl AxisCluster {
    pub fn push(&mut self, x: u32, y: u32) {
        self.points.push((x, y));
    }

    pub fn points_ref(&self) -> &[(u32, u32)] {
        &self.points
    }

    pub fn into_approx_rect(self) -> ApproxRect {
        ApproxRect::new(self.points)
    }
}

pub struct AxisClusterComputer;

pub fn x_axis_selector(point: &(u32, u32)) -> &u32 {
    &point.0
}

pub fn y_axis_selector(point: &(u32, u32)) -> &u32 {
    &point.1
}

impl AxisClusterComputer {
    fn sort_points<F>(points: &[(u32, u32)], selector: F) -> Vec<(u32, u32)>
    where
        F: Fn(&(u32, u32)) -> &u32,
    {
        let mut points = points.to_vec();
        points.sort_by(|a, b| selector(a).cmp(selector(b)));
        points
    }

    pub fn cluster_by_distance<F>(
        points: &[(u32, u32)],
        distance: u32,
        selector: F,
    ) -> Vec<AxisCluster>
    where
        F: Fn(&(u32, u32)) -> &u32,
    {
        let points = Self::sort_points(points, &selector);
        let mut clusters = vec![AxisCluster::default()];
        for point in points {
            let (x, y) = point;
            let last_cluster = clusters.last_mut().unwrap();
            if last_cluster.points.is_empty() {
                last_cluster.push(x, y);
                continue;
            }
            let last_single = *selector(last_cluster.points.last().unwrap());
            if selector(&point).abs_diff(last_single) <= distance {
                last_cluster.push(x, y);
            } else {
                clusters.push(AxisCluster::default());
                clusters.last_mut().unwrap().push(x, y);
            }
        }
        match &clusters[..] {
            [n] if n.points.is_empty() => Vec::default(),
            _ => clusters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_by_distance() {
        let distance = 5;
        let points = [(0, 0), (9, 1), (5, 1), (15, 5), (17, 3)];
        let clusters = AxisClusterComputer::cluster_by_distance(&points, distance, x_axis_selector);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].points, [(0, 0), (5, 1), (9, 1)]);
        assert_eq!(clusters[1].points, [(15, 5), (17, 3)]);
    }
}
