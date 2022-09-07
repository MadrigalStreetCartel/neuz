use std::{fmt, time::Instant};

use crate::image_analyzer::{Color, ImageAnalyzer};

use super::PointCloud;

#[derive(Debug, Default, Clone, Copy)]
pub enum PixelDetectionKind {
    #[default]
    CursorType,
    IsNpc,
}
impl fmt::Display for PixelDetectionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PixelDetectionKind::CursorType => write!(f, "cursor type"),
            PixelDetectionKind::IsNpc => write!(f, "is NPC"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PixelDetectionConfig {
    pub max_x: u32,
    pub max_y: u32,
    pub min_x: u32,
    pub min_y: u32,
    pub refs: Vec<Color>,
}

impl PixelDetectionConfig {
    pub fn new(color: [u8; 3]) -> Self {
        Self {
            refs: vec![Color::new(color[0], color[1], color[2])],
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

                cursor_type.min_x = 0;
                cursor_type.min_y = 0;

                cursor_type.max_x = 1;
                cursor_type.max_y = 1;

                cursor_type
            },
            IsNpc => {
                let mut is_npc = PixelDetectionConfig::new([72, 78, 166]);

                is_npc.min_x = 310;
                is_npc.min_y = 30;

                is_npc.max_x = 1100;
                is_npc.max_y = 60;

                is_npc
            }
        }
    }
}

impl Default for PixelDetectionConfig {
    fn default() -> Self {
        Self {
            max_x: 310,
            max_y: 120,
            min_x: 0,
            min_y: 0,
            refs: vec![Color::default()],
        }
    }
}

impl PartialEq for PixelDetectionConfig {
    fn eq(&self, other: &Self) -> bool {
        /* self.refs == other.refs &&*/
        self.max_x == other.max_x && self.min_x == other.min_x && self.min_y == other.min_y
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PixelDetection {
    pub value: bool,
    pub pixel_kind: PixelDetectionKind,
    pub last_value: bool,
    pub last_update_time: Option<Instant>,
}

impl PartialEq for PixelDetection {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for PixelDetection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl PixelDetection {
    pub fn new(pixel_kind: PixelDetectionKind, image: Option<&ImageAnalyzer>) -> Self {
        let mut res = Self {
            value: false,
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
        let config: PixelDetectionConfig = self.pixel_kind.into();

        let recv = image.pixel_detection(
            config.refs,
            config.min_x,
            config.min_y,
            config.max_x,
            config.max_y,
            Some(10)
        );

        // Receive points from channel
        let cloud = {
            let mut cloud = PointCloud::default();
            while let Ok(point) = recv.recv() {
                cloud.push(point);
            }
            cloud
        };

        let updated_value = !cloud.is_empty();

        // Update values if needed
        if updated_value != self.value {
            self.value = updated_value;
            self.last_update_time = Some(Instant::now());
        }
    }
}
