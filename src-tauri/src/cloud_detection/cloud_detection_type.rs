use crate::data::ColorDetection;

use super::CloudDetectionKind;

#[derive(Debug, Copy, PartialEq, Eq, Hash, Clone)]
pub enum CloudDetectionType {
    Text(CloudDetectionKind),
    Shape(CloudDetectionKind),
    Stat(CloudDetectionKind),
}

impl CloudDetectionType {
    pub fn get_colors(&self) -> ColorDetection {
        match self {
            Self::Text(t) | Self::Stat(t) | Self::Shape(t) => t.get_colors(),
        }
    }
}
