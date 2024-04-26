use serde::{Deserialize, Serialize};

use super::Point;

/// A bounding box in 2D space.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Bounds {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn get_lowest_center_point(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h)
    }

    /// Get the size in square pixels.
    #[inline]
    pub fn size(&self) -> usize {
        (self.w as usize) * (self.h as usize)
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

    #[inline]
    pub fn contains_bounds(&self, other: &Bounds) -> bool {
        other.x >= self.x
            && other.x + other.w <= self.x + self.w
            && other.y >= self.y
            && other.y + other.h <= self.y + self.h
    }

    /// Merge ONLY if the given bounds is directly inside the bounds.
    #[inline]
    pub fn merge_bounds(&self, other: &Bounds) -> Option<Bounds> {
        let self_is_bigger = self.size() > other.size();
        let other_in_self = self.contains_bounds(other);
        let self_in_other = {
            if other_in_self {
                false
            } else {
                other.contains_bounds(self)
            }
        };

        if self_in_other {
            Some(*self)
        } else if other_in_self {
            Some(*other)
        } else {
            None
        }
    }

    /// Returns point of corners of the bounds.
    #[inline]
    #[allow(dead_code)]
    pub fn corners(&self) -> [Point; 4] {
        [
            Point::new(self.x, self.y),
            Point::new(self.x + self.w, self.y),
            Point::new(self.x, self.y + self.h),
            Point::new(self.x + self.w, self.y + self.h),
        ]
    }

    /// Returns pixel detection area.
    #[inline]
    #[allow(dead_code)]
    pub fn to_detection_area(&self) -> [u32; 4] {
        [self.x, self.y, self.x + self.w, self.y + self.h]
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
