use serde::{Deserialize, Serialize};

use super::Point;

/// A bounding box in 2D space.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Bounds {
    pub fn get_lowest_center_point(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h)
    }

    /// Get the size in square pixels.
    #[inline]
    pub fn size(&self) -> usize {
        self.w as usize * self.h as usize
    }

    /// Expand the bounds in all directions by the given amount.
    #[allow(dead_code)]
    pub fn grow_by(&self, px: u32) -> Bounds {
        Bounds {
            x: self.x.saturating_sub(px / 2),
            y: self.y.saturating_sub(px / 2),
            w: self.w + px,
            h: self.h + px,
        }
    }

    /// Get the center point.
    #[inline]
    #[allow(dead_code)]
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    /// Check if the given point is inside the bounds.
    #[inline]
    #[allow(dead_code)]
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }
}

impl slog::Value for Bounds {
    fn serialize(
        &self,
        _record: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self))
    }
}
