use std::sync::mpsc::sync_channel;

use libscreenshot::ImageBuffer;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    data::{point_selector, Bounds, MobType, Point, PointCloud, Target, TargetType},
    platform::{IGNORE_AREA_BOTTOM, IGNORE_AREA_TOP},
    utils::Timer,
};
#[derive(Debug)]
pub enum StatusBarKind {
    Hp,
    Mp,
    Fp,
    Xp,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StatInfo {
    pub max_w: u32,
    pub value: u32,
}

impl PartialEq for StatInfo {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for StatInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}
#[derive(Debug, Clone, Copy)]
pub struct StatusBarConfig {
    pub max_search_x: u32,
    pub max_search_y: u32,
    pub refs: [[u8; 3]; 4],
}

impl StatusBarConfig {
    pub fn new(colors: [[u8; 3]; 4]) -> Self {
        Self {
            refs: colors,
            ..Default::default()
        }
    }
}

impl From<StatusBarKind> for StatusBarConfig {
    fn from(kind: StatusBarKind) -> Self {
        use StatusBarKind::*;

        match kind {
            Hp => {
                StatusBarConfig::new([[174, 18, 55], [188, 24, 62], [204, 30, 70], [220, 36, 78]])
            }
            Mp => StatusBarConfig::new([
                [20, 84, 196],
                [36, 132, 220],
                [44, 164, 228],
                [56, 188, 232],
            ]),
            Fp => {
                StatusBarConfig::new([[45, 230, 29], [28, 172, 28], [44, 124, 52], [20, 146, 20]])
            }
            Xp => StatusBarConfig::new([
                [48, 185, 244],
                [128, 212, 245],
                [52, 196, 252],
                [92, 236, 252],
            ]),
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            max_search_x: 310,
            max_search_y: 120,
            refs: [[0; 3]; 4],
        }
    }
}

impl PartialEq for StatusBarConfig {
    fn eq(&self, other: &Self) -> bool {
        self.refs == other.refs && self.max_search_x == other.max_search_x
    }
}

const HP_BAR:Bar = Bar{max_search_x:310,max_search_y:120,refs:[[174, 18, 55], [188, 24, 62], [204, 30, 70], [220, 36, 78]]};
const MP_BAR:Bar = Bar{max_search_x:310,max_search_y:120,refs:[[20, 84, 196], [36, 132, 220], [44, 164, 228], [56, 188, 232]]};
const FP_BAR:Bar = Bar{max_search_x:310,max_search_y:120,refs:[[45, 230, 29], [28, 172, 28], [44, 124, 52], [20, 146, 20]]};
const EXP_BAR:Bar = Bar{max_search_x:310,max_search_y:120,refs:[[48, 185, 244], [128, 212, 245], [52, 196, 252], [92, 236, 252]]};
pub struct ImageAnalyzer {
    image: ImageBuffer,
}

impl ImageAnalyzer {
    pub fn new(image: ImageBuffer) -> Self {
        Self { image }
    }

    fn merge_cloud_into_mobs(
        cloud: &PointCloud,
        mob_type: TargetType,
        ignore_size: bool,
    ) -> Vec<Target> {
        let _timer = Timer::start_new("merge_cloud_into_mobs");

        // Max merge distance
        let max_distance_x: u32 = 30;
        let max_distance_y: u32 = 5;

        // Cluster coordinates in x-direction
        let x_clusters = cloud.cluster_by_distance(max_distance_x, point_selector::x_axis);

        let mut xy_clusters = Vec::default();
        for x_cluster in x_clusters {
            // Cluster current x-cluster coordinates in y-direction
            let local_y_clusters =
                x_cluster.cluster_by_distance(max_distance_y, point_selector::y_axis);
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
                if ignore_size {
                    true
                } else {
                    // Filter out small clusters (likely to cause misclicks)
                    mob.bounds.w > 30
                    // Filter out huge clusters (likely to be Violet Magician Troupe)
                    && mob.bounds.size() < (220 * 6)
                }
            })
            .collect()
    }

    /// Check if pixel `c` matches reference pixel `r` with the given `tolerance`.
    #[inline(always)]
    fn pixel_matches(c: &[u8; 4], r: &[u8; 3], tolerance: u8) -> bool {
        let matches_inner = |a: u8, b: u8| match (a, b) {
            (a, b) if a == b => true,
            (a, b) if a > b => a.saturating_sub(b) <= tolerance,
            (a, b) if a < b => b.saturating_sub(a) <= tolerance,
            _ => false,
        };
        let perm = [(c[0], r[0]), (c[1], r[1]), (c[2], r[2])];
        perm.iter().all(|&(a, b)| matches_inner(a, b))
    }

    pub fn identify_mobs(&self) -> Vec<Target> {
        let _timer = Timer::start_new("identify_mobs");

        // Create collections for passive and aggro mobs
        let mut mob_coords_pas: Vec<Point> = Vec::default();
        let mut mob_coords_agg: Vec<Point> = Vec::default();

        // Reference colors
        let ref_color_pas: [u8; 3] = [0xe8, 0xe8, 0x94]; // Passive mobs
        let ref_color_agg: [u8; 3] = [0xd3, 0x0f, 0x0d]; // Aggro mobs

        // Collect pixel clouds
        struct MobPixel(u32, u32, TargetType);
        let (snd, recv) = sync_channel::<MobPixel>(4096);
        self.image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP || y > self.image.height() - IGNORE_AREA_BOTTOM {
                    return;
                }
                for (x, _, px) in row {
                    if px.0[3] != 255 || y > self.image.height() - IGNORE_AREA_BOTTOM {
                        return;
                    }
                    if Self::pixel_matches(&px.0, &ref_color_pas, 2) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Passive))));
                    } else if Self::pixel_matches(&px.0, &ref_color_agg, 8) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Aggressive))));
                    }
                }
            });
        while let Ok(px) = recv.recv() {
            match px.2 {
                TargetType::Mob(MobType::Passive) => mob_coords_pas.push(Point::new(px.0, px.1)),
                TargetType::Mob(MobType::Aggressive) => mob_coords_agg.push(Point::new(px.0, px.1)),
                _ => unreachable!(),
            }
        }

        // Categorize mobs
        let mobs_pas = Self::merge_cloud_into_mobs(
            &PointCloud::new(mob_coords_pas),
            TargetType::Mob(MobType::Passive),
            false,
        );
        let mobs_agg = Self::merge_cloud_into_mobs(
            &PointCloud::new(mob_coords_agg),
            TargetType::Mob(MobType::Aggressive),
            false,
        );

        // Return all mobs
        Vec::from_iter(mobs_agg.into_iter().chain(mobs_pas.into_iter()))
    }

    pub fn identify_target_marker(&self) -> Option<Target> {
        let _timer = Timer::start_new("identify_target_marker");
        let mut coords = Vec::default();

        // Reference color
        let ref_color: [u8; 3] = [246, 90, 106];

        // Collect pixel clouds
        let (snd, recv) = sync_channel::<Point>(4096);
        self.image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP || y > self.image.height() - IGNORE_AREA_BOTTOM {
                    return;
                }
                for (x, _, px) in row {
                    if px.0[3] != 255 {
                        return;
                    }
                    if Self::pixel_matches(&px.0, &ref_color, 2) {
                        #[allow(clippy::drop_copy)] // this is fine
                        drop(snd.send(Point::new(x, y)));
                    }
                }
            });

        // Receive points from channel
        while let Ok(point) = recv.recv() {
            coords.push(point);
        }

        // Identify target marker entities
        let target_markers =
            Self::merge_cloud_into_mobs(&PointCloud::new(coords), TargetType::TargetMarker, true);

        // Find biggest target marker
        target_markers.into_iter().max_by_key(|x| x.bounds.size())
    }

    /// Distance: `[0..=500]`
    pub fn find_closest_mob<'a>(
        &self,
        mobs: &'a [Target],
        avoid_bounds: Option<&Bounds>,
        max_distance: i32,
    ) -> Option<&'a Target> {
        let _timer = Timer::start_new("find_closest_mob");

        // Calculate middle point of player
        let mid_x = (self.image.width() / 2) as i32;
        let mid_y = (self.image.height() / 2) as i32;

        // Calculate 2D euclidian distances to player
        let mut distances = Vec::default();
        for mob in mobs {
            let point = mob.get_attack_coords();
            let distance = (((mid_x - point.x as i32).pow(2) + (mid_y - point.y as i32).pow(2))
                as f64)
                .sqrt() as i32;
            distances.push((mob, distance));
        }

        // Sort by distance
        distances.sort_by_key(|&(_, distance)| distance);

        // Remove mobs that are too far away
        distances = distances
            .into_iter()
            .filter(|&(_, distance)| distance <= max_distance)
            .collect();

        if let Some(_) = avoid_bounds {
            // Try finding closest mob that's not the mob to be avoided
            if let Some((mob, _distance)) = distances.iter().find(|(_mob, distance)| {
                *distance > 150
                // let coords = mob.name_bounds.get_lowest_center_point();
                // !avoid_bounds.grow_by(100).contains_point(&coords) && *distance > 200
            }) {
                Some(mob)
            } else {
                None
            }
        } else {
            // Return closest mob
            if let Some((mob, _distance)) = distances.first() {
                Some(*mob)
            } else {
                None
            }
        }
    }

    pub fn detect_status_bar(
        &self,
        last_stats: StatInfo,
        status_bar: StatusBarKind,
    ) -> Option<StatInfo> {
        let status_bar_config: StatusBarConfig = status_bar.into();
        let (snd, recv) = sync_channel::<Point>(4096);
        self.image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                // Skip this row if it's in an ignored area
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP
                    || y > self.image.height() - IGNORE_AREA_BOTTOM
                    || y > IGNORE_AREA_TOP + status_bar_config.max_search_y
                {
                    return;
                }

                // Loop over columns
                'outer: for (x, _, px) in row {
                    if px.0[3] != 255 || x >= status_bar_config.max_search_x {
                        return;
                    }
                    for ref_color in status_bar_config.refs.iter() {
                        // Check if the pixel matches any of the reference colors
                        if Self::pixel_matches(&px.0, &ref_color, 5) {
                            #[allow(clippy::drop_copy)]
                            drop(snd.send(Point::new(x, y)));

                            // Continue to next column
                            continue 'outer;
                        }
                    }
                }
            });

        // Receive points from channel
        let cloud = {
            let mut cloud = PointCloud::default();
            while let Ok(point) = recv.recv() {
                cloud.push(point);
            }
            cloud
        };

        // Calculate bounds
        let bounds = cloud.to_bounds();

        // Recalculate value tracking info
        let max_w = bounds.w.max(last_stats.max_w);
        let value_frac = bounds.w as f32 / max_w as f32;
        let value_scaled = ((value_frac * 100_f32) as u32).max(0).min(100);
        let value = StatInfo {
            max_w,
            value: value_scaled,
        };

        Some(value)
    }
}
