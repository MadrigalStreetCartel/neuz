use super::{ Bounds, Point };

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MobType {
    Passive,
    Aggressive,
    Violet,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TargetType {
    Mob(MobType),
    #[default]
    TargetMarker,
}

/// A target in 2D space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Target {
    pub target_type: TargetType,
    pub bounds: Bounds,
}

impl Target {
    /// Get the approximated attack coordinates.
    pub fn get_attack_coords(&self) -> Point {
        let point = self.bounds.get_lowest_center_point();
        Point::new(point.x, point.y + 10)
    }

    pub fn get_target_distance_to(&self, point: Point) -> i32 {
        // Calculate middle point of player
        let mid_x = point.x as i32;
        let mid_y = point.y as i32;

        // Calculate 2D euclidian distances to player
        let point = self.bounds.get_lowest_center_point();

        (
            ((mid_x - (point.x as i32)).pow(2) + (mid_y - (point.y as i32)).pow(2)) as f64
        ).sqrt() as i32
    }
}
