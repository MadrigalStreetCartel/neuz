use super::{Bounds, Point};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MobType {
    Passive,
    Aggressive,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum _MobRank {
    Small,
    Normal,
    Captain,
    GiantOrViolet,
    Boss,
    Insane
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetMarkerType {
    Passive,
    Aggressive,
    _Flying,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetType {
    Mob(MobType),
    TargetMarker(TargetMarkerType),
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::TargetMarker(TargetMarkerType::Passive)
    }
}

/// A target in 2D space.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
