use super::{Bounds, Point};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MobType {
    Passive,
    Aggressive,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TargetType {
    Mob(MobType),
    #[default]
    TargetMarker,
}


pub enum DirectionType {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,

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

    /// Get target distance
    pub fn get_distance(&self) -> i32 {

        // Calculate middle point of player
        let mid_x = (800 / 2) as i32;
        let mid_y = (600 / 2) as i32;
        let point = self.get_attack_coords();

        // Calculate 2D euclidian distances to player
        let distance = (((mid_x - point.x as i32).pow(2) + (mid_y - point.y as i32).pow(2))
            as f64)
            .sqrt() as i32;
        distance
    }

    /// Get target direction 0 is current direction, 1 is opposite
    pub fn get_directions(&self) -> (DirectionType, DirectionType) {
        let click_coord = self.get_attack_coords();
        let x = click_coord.x;
        let y = click_coord.y;
        let half_width = 800 / 2;
        let half_height = 600 / 2;
        if x >= half_width {
            if y >= half_height {
                (DirectionType::BottomRight, DirectionType::TopLeft)
            } else {
                (DirectionType::TopRight, DirectionType::BottomLeft)
            }
        } else {
            if y >= half_height {
                (DirectionType::BottomLeft, DirectionType::TopRight)
            } else {
                (DirectionType::TopLeft, DirectionType::BottomRight)
            }
        }
    }

    /// Get the approximated opposite movement
    pub fn  get_active_avoid_coords(&self, move_distance: u32) -> Point {
        let half_width = 800 / 2;
        let half_height = 600 / 2;
        let opposite_direction = self.get_directions().1;
        return {
            match opposite_direction {
                DirectionType::TopLeft => Point::new( half_width - move_distance,  half_height - move_distance),
                DirectionType::BottomLeft => Point::new( half_width - move_distance,  half_height + move_distance),
                DirectionType::TopRight => Point::new( half_width + move_distance,  half_height - move_distance),
                DirectionType::BottomRight => Point::new( half_width + move_distance,  half_height + move_distance),
            }
        }
    }
}
