use crate::{image_analyzer::ImageAnalyzer, utils::Timer};

use super::{
    bounds, point_selector, CloudDetectionCategorie, CloudDetectionKind, ColorDetection, MobType, PointCloud, Target, TargetType
};

#[derive(Debug, PartialEq, Eq)]
pub struct CloudDetectionConfig {
    pub detector: CloudDetectionCategorie,
    pub bounds: bounds::Bounds,
    pub color_detection: ColorDetection,
}

impl CloudDetectionConfig {
    pub fn new(detector: CloudDetectionCategorie) -> Self {
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

impl Clone for CloudDetectionConfig {
    fn clone(&self) -> Self {
        CloudDetectionConfig::new(self.detector.clone())
    }
}

impl Default for CloudDetectionConfig {
    fn default() -> Self {
        Self {
            detector: CloudDetectionCategorie::None,
            bounds: bounds::Bounds::default(),
            color_detection: ColorDetection::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CloudDetection {
    pub kind: CloudDetectionCategorie,
    pub cloud: PointCloud,
}

impl CloudDetection {
    pub fn _new(kind: CloudDetectionCategorie, cloud: PointCloud) -> Self {
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
        let target_markers = Self::merge_cloud_into_mobs(
            &self.cloud,
            TargetType::TargetMarker
        );

        // Find biggest target marker
        target_markers.into_iter().max_by_key(|x| x.bounds.size())
    }

    pub fn process_mobs(&self) -> Vec<Target> {
        let mob_type: TargetType = {
            match self.kind {
                CloudDetectionCategorie::Mover(CloudDetectionKind::Mob(t)) => TargetType::Mob(t),
                _ => TargetType::Mob(MobType::Aggressive),
            }
        };

        if mob_type == TargetType::TargetMarker {
            return vec![];
        }
        let mob_markers = Self::merge_cloud_into_mobs(&self.cloud, mob_type);

        mob_markers
            .into_iter()
            .map(|x| x.into())
            .collect()
    }

    pub fn merge_cloud_into_mobs(
        cloud: &PointCloud,
        mob_type: TargetType //ignore_size: bool,
    ) -> Vec<Target> {
        let _timer = Timer::start_new("merge_cloud_into_mobs");

        // Max merge distance
        let max_distance_x: u32 = 50;
        let max_distance_y: u32 = 3;

        // Cluster coordinates in x-direction
        let x_clusters = cloud.cluster_by_distance(max_distance_x, point_selector::x_axis);

        let mut xy_clusters = Vec::default();
        for x_cluster in x_clusters {
            // Cluster current x-cluster coordinates in y-direction
            let local_y_clusters = x_cluster.cluster_by_distance(
                max_distance_y,
                point_selector::y_axis
            );
            // Extend final xy-clusters with local y-clusters
            xy_clusters.extend(local_y_clusters.into_iter());
        }

        // Create mobs from clusters
        xy_clusters
            .into_iter()
            .map(|cluster| Target {
                target_type: mob_type,
                bounds: cluster.to_bounds(),
            })
            .filter(|mob| {
                if mob_type == TargetType::TargetMarker {
                    return true;
                }
                // Filter out small clusters (likely to cause misclicks)
                mob.bounds.w > 11 &&
                    // Filter out huge clusters (likely to be Violet Magician Troupe)
                    mob.bounds.w < 180
            })
            .collect()
    }
}
