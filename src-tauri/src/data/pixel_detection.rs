use std::{time::Instant, fmt};

use crate::image_analyzer::ImageAnalyzer;

#[derive(Debug, Default, Clone, Copy)]
pub enum PixelDetectionKind {
    #[default]
    CursorType,

}
impl fmt::Display for PixelDetectionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PixelDetectionKind::CursorType => write!(f, "cursor type"),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct PixelDetectionConfig {
    pub max_search_x: u32,
    pub max_search_y: u32,
    pub min_search_x: u32,
    pub min_search_y: u32,
    pub refs: [u8; 3],
}

impl PixelDetectionConfig {
    pub fn new(color: [u8; 3]) -> Self {
        Self {
            refs: color,
            ..Default::default()
        }
    }
}

impl From<PixelDetectionKind> for PixelDetectionConfig {
    fn from(kind: PixelDetectionKind) -> Self {
        use PixelDetectionKind::*;

        match kind {
            CursorType => {
                let mut cursor_type = PixelDetectionConfig::new([0, 128, 0]);
                cursor_type.min_search_x = 0;
                cursor_type.min_search_y = 0;

                cursor_type.max_search_x = 10;
                cursor_type.max_search_y = 10;

                cursor_type
            }
        }
    }
}

impl Default for PixelDetectionConfig {
    fn default() -> Self {
        Self {
            max_search_x: 310,
            max_search_y: 120,
            min_search_x: 0,
            min_search_y: 0,
            refs: [0; 3],
        }
    }
}

impl PartialEq for PixelDetectionConfig {
    fn eq(&self, other: &Self) -> bool {
        self.refs == other.refs && self.max_search_x == other.max_search_x
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PixelDetectionInfo {
    pub value: bool,
    pub pixel_kind: PixelDetectionKind,
    pub last_value: bool,
    pub last_update_time: Option<Instant>,
}

impl PartialEq for PixelDetectionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for PixelDetectionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl PixelDetectionInfo {
    pub fn new(
        value: bool,
        pixel_kind: PixelDetectionKind,
        image: Option<&ImageAnalyzer>,
    ) -> Self {
        let mut res = Self {
            value,
            pixel_kind,
            last_update_time: Some(Instant::now()),
            last_value: false,
        };
        if image.is_some() {
            res.update_value(image.unwrap());
        }
        res
    }

    pub fn update_value(&mut self, image: &ImageAnalyzer) {
        let updated_value = image.detect_pixel(*self);
        let old_value = self.value;

        if updated_value != old_value {

            // Update values
            self.value = updated_value;
            self.last_update_time = Some(Instant::now());
        }
    }
}

