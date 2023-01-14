use std::{
    sync::mpsc::{sync_channel, Receiver},
    time::Instant,
};

//use libscreenshot::shared::Area;
use libscreenshot::{ImageBuffer, WindowCaptureProvider};
use rayon::iter::{ParallelBridge, ParallelIterator};
use slog::Logger;
use tauri::{Window};

use crate::{
    data::{point_selector, Bounds, ClientStats, MobType, Point, PointCloud, Target, TargetType},
    ipc::FarmingConfig,
    platform::{IGNORE_AREA_BOTTOM, IGNORE_AREA_TOP},
    utils::Timer,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub refs: [u8; 3],
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { refs: [r, g, b] }
    }
}

#[derive(Debug, Clone)]
pub struct ImageAnalyzer {
    image: Option<ImageBuffer>,
    pub window_id: u64,
    pub client_stats: ClientStats,
}

impl ImageAnalyzer {
    pub fn new(window: &Window) -> Self {
        Self {
            window_id: 0,
            image: None,
            client_stats: ClientStats::new(window.to_owned()),
        }
    }

    pub fn image_is_some(&self) -> bool {
        self.image.is_some()
    }

    pub fn capture_window(&mut self, logger: &Logger, _config: &FarmingConfig) {
        let _timer = Timer::start_new("capture_window");
        if self.window_id == 0 {
            return;
        }

        if let Some(provider) = libscreenshot::get_window_capture_provider() {
            if let Ok(image) = provider.capture_window(self.window_id) {
                self.image = Some(image);
            } else {
                slog::warn!(logger, "Failed to capture window"; "window_id" => self.window_id);
            }
        }
    }

    pub fn pixel_detection(
        &self,
        colors: Vec<Color>,
        min_x: u32,
        min_y: u32,
        mut max_x: u32,
        mut max_y: u32,
        tolerence: Option<u8>,
    ) -> Receiver<Point> {
        let (snd, recv) = sync_channel::<Point>(4096);
        let image = self.image.as_ref().unwrap();

        if max_x == 0 {
            max_x = image.width();
        }

        if max_y == 0 {
            max_y = image.height();
        }

        image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                // Skip this row if it's in an ignored area
                let image_height = image.height();
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP
                    || y > image_height
                        .checked_sub(IGNORE_AREA_BOTTOM)
                        .unwrap_or(image_height)
                    || y > IGNORE_AREA_TOP + max_y
                    || y > max_y
                    || y < min_y
                {
                    return;
                }

                // Loop over columns
                'outer: for (x, _, px) in row {
                    if px.0[3] != 255 || x >= max_x {
                        return;
                    } else if x < min_x {
                        continue;
                    }

                    for ref_color in colors.iter() {
                        // Check if the pixel matches any of the reference colors
                        if Self::pixel_matches(&px.0, &ref_color.refs, tolerence.unwrap_or(5)) {
                            #[allow(clippy::drop_copy)]
                            drop(snd.send(Point::new(x, y)));

                            // Continue to next column
                            continue 'outer;
                        }
                    }
                }
            });
        recv
    }

    fn merge_cloud_into_mobs(
        config: Option<&FarmingConfig>,
        cloud: &PointCloud,
        mob_type: TargetType,
        //ignore_size: bool,
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
                if let Some(config) = config {
                    // Filter out small clusters (likely to cause misclicks)
                    mob.bounds.w > config.min_mobs_name_width()
                    // Filter out huge clusters (likely to be Violet Magician Troupe)
                    && mob.bounds.w < config.max_mobs_name_width()
                } else {
                    true
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

    pub fn identify_mobs(&self, config: &FarmingConfig) -> Vec<Target> {
        let _timer = Timer::start_new("identify_mobs");

        // Create collections for passive and aggro mobs
        let mut mob_coords_pas: Vec<Point> = Vec::default();
        let mut mob_coords_agg: Vec<Point> = Vec::default();

        // Reference colors
        let ref_color_pas_wrapped: [Option<u8>; 3] = config.passive_mobs_colors(); // Passive mobs 234, 234, 149
        let ref_color_agg_wrapped: [Option<u8>; 3] = config.aggressive_mobs_colors(); // Aggro mobs 179, 23, 23
        let ref_color_pas: [u8; 3] = [
            ref_color_pas_wrapped[0].unwrap_or(234),
            ref_color_pas_wrapped[1].unwrap_or(234),
            ref_color_pas_wrapped[2].unwrap_or(149),
        ];
        let ref_color_agg: [u8; 3] = [
            ref_color_agg_wrapped[0].unwrap_or(179),
            ref_color_agg_wrapped[1].unwrap_or(23),
            ref_color_agg_wrapped[2].unwrap_or(23),
        ];

        // Collect pixel clouds
        struct MobPixel(u32, u32, TargetType);
        let (snd, recv) = sync_channel::<MobPixel>(4096);
        let image = self.image.as_ref().unwrap();
        image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP || y > image.height() - IGNORE_AREA_BOTTOM {
                    return;
                }
                for (x, _, px) in row {
                    if px.0[3] != 255 {
                        return;
                    } else if x <= 250 && y <= 110 {
                        // avoid detect the health bar as a monster
                        continue;
                    }
                    if Self::pixel_matches(&px.0, &ref_color_pas, config.passive_tolerence()) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Passive))));
                    } else if Self::pixel_matches(
                        &px.0,
                        &ref_color_agg,
                        config.aggressive_tolerence(),
                    ) {
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
            Some(config),
            &PointCloud::new(mob_coords_pas),
            TargetType::Mob(MobType::Passive),
        );
        let mobs_agg = Self::merge_cloud_into_mobs(
            Some(config),
            &PointCloud::new(mob_coords_agg),
            TargetType::Mob(MobType::Aggressive),
        );

        // Return all mobs
        Vec::from_iter(mobs_agg.into_iter().chain(mobs_pas.into_iter()))
    }

    pub fn identify_target_marker(&self, blank_target: bool) -> Option<Target> {
        let _timer = Timer::start_new("identify_target_marker");
        let mut coords = Vec::default();

        // Reference color
        let ref_color: Color = {
            if !blank_target {
                Color::new(246, 90, 106)
            } else {
                Color::new(164, 180, 226)
            }
        };


        // Collect pixel clouds
        let recv = self.pixel_detection(vec![ref_color], 0, 0, 0, 0, None);

        // Receive points from channel
        while let Ok(point) = recv.recv() {
            coords.push(point);
        }

        // Identify target marker entities
        let target_markers = Self::merge_cloud_into_mobs(
            None,
            &PointCloud::new(coords),
            TargetType::TargetMarker,
        );

        if !blank_target && target_markers.is_empty() {
            return self.identify_target_marker(true);
        }

        // Find biggest target marker
        target_markers.into_iter().max_by_key(|x| x.bounds.size())
    }
    pub fn get_target_marker_distance(&self,  mob: Target) -> i32 {
        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = (image.width() / 2) as i32;
        let mid_y = (image.height() / 2) as i32;

        // Calculate 2D euclidian distances to player
        let point = mob.get_attack_coords();
        let distance = (((mid_x - point.x as i32).pow(2) + (mid_y - point.y as i32).pow(2))
            as f64)
            .sqrt() as i32;
        distance
    }
    /// Distance: `[0..=500]`
    pub fn find_closest_mob<'a>(
        &self,
        mobs: &'a [Target],
        //avoid_bounds: Option<&Bounds>,
        avoid_list: Option<&Vec<(Bounds, Instant, u128)>>,
        max_distance: i32,
        _logger: &Logger,
    ) -> Option<&'a Target> {
        let _timer = Timer::start_new("find_closest_mob");

        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = (image.width() / 2) as i32;
        let mid_y = (image.height() / 2) as i32;

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

        if let Some(avoided_bounds) = avoid_list {
            // Try finding closest mob that's not the mob to be avoided
            if let Some((mob, _distance)) = distances.iter().find(|(mob, _distance)| {
                //*distance > 55
                let coords = mob.get_attack_coords();
                let mut result = true;
                for avoided_item in avoided_bounds {
                    if avoided_item.0.contains_point(&coords) {
                        //slog::debug!(logger, ""; "Avoided bounds" => avoided_item.0);
                        result = false;
                        break;
                    }
                }
                result // && *distance > 20
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
}
