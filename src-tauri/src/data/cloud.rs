use crate::image_analyzer::ImageAnalyzer;

use super::{
    bounds, ColorDetection, MobType, PixelCloudKind, PixelCloudKindCategorie, PointCloud, Target, TargetType
};

#[derive(Debug, PartialEq, Eq)]
pub struct PixelCloudConfig {
    pub detector: PixelCloudKindCategorie,
    pub bounds: bounds::Bounds,
    pub color_detection: ColorDetection,
}

impl PixelCloudConfig {
    pub fn new(detector: PixelCloudKindCategorie) -> Self {
        let mut result = Self {
            detector,
            bounds: detector.get_bounds(),
            color_detection: detector.get_colors().expect("No color detection found"),
        };

        if !result.is_valid() {
            result = Self::default();
        }
        result
    }

    fn is_valid(&self) -> bool {
        self.bounds.is_valid()
    }

    fn is_within_x_bounds(&self, x: u32) -> bool {
        x >= self.bounds.x && x <= self.bounds.x + self.bounds.w
    }

    fn is_within_y_bounds(&self, y: u32) -> bool {
        y >= self.bounds.y && y <= self.bounds.y + self.bounds.h
    }

    pub fn is_within_bounds(&self, x: u32, y: u32) -> bool {
        self.is_within_x_bounds(x) && self.is_within_y_bounds(y)
    }

    pub fn pixel_compare(&self, color: &[u8; 4]) -> bool {
        let matched = self.color_detection.color_match(color);
        matched
    }
}

impl Clone for PixelCloudConfig {
    fn clone(&self) -> Self {
        PixelCloudConfig::new(self.detector.clone())
    }
}

impl Default for PixelCloudConfig {
    fn default() -> Self {
        Self {
            detector: PixelCloudKindCategorie::None,
            bounds: bounds::Bounds::default(),
            color_detection: ColorDetection::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PixelCloud {
    pub kind: PixelCloudKindCategorie,
    pub cloud: PointCloud,
}

impl PixelCloud {
    pub fn _new(kind: PixelCloudKindCategorie, cloud: PointCloud) -> Self {
        Self {
            kind,
            cloud,
        }
    }

    pub fn process_stats(&self, current_max_w: u32) -> (u32, u32) {
        if self.cloud.is_empty() {
            return (0, current_max_w);
        }
        // Calculate bounds
        let bounds = self.cloud.to_bounds();

        // Recalculate value tracking info
        let updated_max_w = bounds.w.max(current_max_w);
        let value_frac = (bounds.w as f32) / (updated_max_w as f32);
        let updated_value = ((value_frac * 100_f32) as u32).max(0).min(100);

        (updated_value, updated_max_w)
    }

    pub fn process_target(&self) -> Option<Target> {
        let target_markers = ImageAnalyzer::merge_cloud_into_mobs(
            None,
            &self.cloud,
            TargetType::TargetMarker
        );

        // Find biggest target marker
        target_markers.into_iter().max_by_key(|x| x.bounds.size())
    }

    pub fn process_mobs(&self) -> Vec<Target> {
        let mob_type: TargetType = {
            match self.kind {
                PixelCloudKindCategorie::Mover(PixelCloudKind::Mob(t)) => TargetType::Mob(t),
                _ => TargetType::Mob(MobType::Aggressive),
            }
        };
        let mob_markers = ImageAnalyzer::merge_cloud_into_mobs(None, &self.cloud, mob_type);

        mob_markers
            .into_iter()
            .map(|x| x.into())
            .collect()
    }
}
