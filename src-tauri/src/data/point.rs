use std::fmt::Display;

/// A point in 2D space.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    /// Construct a new `Point`.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl From<(u32, u32)> for Point {
    fn from(point: (u32, u32)) -> Self {
        Self {
            x: point.0,
            y: point.1,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl slog::Value for Point {
    fn serialize(
        &self,
        _record: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self))
    }
}
