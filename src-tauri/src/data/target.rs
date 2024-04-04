use super::{Bounds, Point};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Default)]
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
}
