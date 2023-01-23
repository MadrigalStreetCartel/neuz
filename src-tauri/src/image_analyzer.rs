use palette::rgb::Rgb;
use palette::{FromColor, Hsv, Srgb};
use std::{
    sync::mpsc::{sync_channel, Receiver},
    time::Instant,
};
//use libscreenshot::shared::Area;
use libscreenshot::{ImageBuffer, WindowCaptureProvider};
use rayon::iter::{ParallelBridge, ParallelIterator};
use slog::Logger;

use crate::data::stats_info::TargetMarkerType;
use crate::{
    data::{point_selector, Bounds, ClientStats, MobType, Point, PointCloud, Target, TargetType},
    ipc::{BotConfig, FarmingConfig},
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

    pub fn to_hsv(&self) -> Hsv {
        let red = self.refs[0];
        let green = self.refs[1];
        let blue = self.refs[2];

        let rgb = Rgb::new(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        );
        Hsv::from_color(rgb)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageAnalyzer {
    image: Option<ImageBuffer>,
    pub window_id: u64,
    pub client_stats: ClientStats,
}

impl ImageAnalyzer {
    pub fn new() -> Self {
        Self {
            window_id: 0,
            image: None,
            client_stats: ClientStats::new(),
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
                self.image = Some(image.clone());
            } else {
                slog::warn!(logger, "Failed to capture window"; "window_id" => self.window_id);
            }
        }
    }

    pub fn pixel_detection(
        &self,
        colors: Vec<Hsv>,
        min_x: u32,
        min_y: u32,
        mut max_x: u32,
        mut max_y: u32,
        tolerence: Option<Hsv>,
    ) -> Receiver<(Point, Hsv)> {
        let (snd, recv) = sync_channel::<(Point, Hsv)>(4096);
        let image = self.image.clone().unwrap();

        if max_x == 0 {
            max_x = image.width();
        }

        if max_y == 0 {
            max_y = image.height();
        }

        let image_height = image.height();

        image.enumerate_rows().par_bridge().for_each(|(y, row)| {
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
                    let current_pixel_color = Color::new(px.0[0], px.0[1], px.0[2]);
                    let default_tol = Hsv::new(1.0, 0.10, 0.3);
                    if Self::pixel_matches_hsv(
                        current_pixel_color,
                        *ref_color,
                        tolerence.unwrap_or(default_tol),
                    ) {
                        #[allow(clippy::drop_copy)]
                        drop(snd.send((Point::new(x, y), current_pixel_color.to_hsv())));

                        // Continue to next column
                        continue 'outer;
                    }
                }
            }
        });
        recv
    }

    fn merge_cloud_into_mobs(
        config: Option<&BotConfig>,
        cloud: &PointCloud,
        mob_type: TargetType,
        //ignore_size: bool,
    ) -> Vec<Target> {
        let _timer = Timer::start_new("merge_cloud_into_mobs");

        // Max merge distance
        let max_distance_x: u32 = 200;
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

    fn pixel_matches_hsv(currentPixelColorRGB: Color, search_hsv: Hsv, tolerance_hsv: Hsv) -> bool {
        let current_pixel_hsv = currentPixelColorRGB.to_hsv();

        let current_pixel_hsv_hue = current_pixel_hsv.hue.to_positive_degrees();
        let search_hsv_hue = search_hsv.hue.to_positive_degrees();
        let tolerance_hsv_hue = tolerance_hsv.hue.to_positive_degrees();
        // HUE
        let hue_too_low = current_pixel_hsv_hue < (search_hsv_hue - tolerance_hsv_hue);
        if hue_too_low {
            return false;
        }

        let hue_too_high = current_pixel_hsv_hue > (search_hsv_hue + tolerance_hsv_hue);
        if hue_too_high {
            return false;
        }

        // Saturation
        let saturation_too_low =
            current_pixel_hsv.saturation < (search_hsv.saturation - tolerance_hsv.saturation);
        if saturation_too_low {
            return false;
        }

        let saturation_too_high =
            current_pixel_hsv.saturation > (search_hsv.saturation + tolerance_hsv.saturation);
        if saturation_too_high {
            return false;
        }

        // Value
        let value_too_low = current_pixel_hsv.value < (search_hsv.value - tolerance_hsv.value);
        if value_too_low {
            return false;
        }

        let value_too_high = current_pixel_hsv.value > (search_hsv.value + tolerance_hsv.value);
        if value_too_high {
            return false;
        }

        // Color match
        return true;
    }

    pub fn identify_mobs(&self, config: &BotConfig) -> Vec<Target> {
        let _timer = Timer::start_new("identify_mobs");

        // Create collections for passive and aggro mobs
        let mut mob_coords_pas: Vec<Point> = Vec::default();
        let mut mob_coords_agg: Vec<Point> = Vec::default();

        // Reference colors
        let ref_color_agg = Hsv::new(0.0, 0.98, 0.918);
        let tol_color_agg = Hsv::new(0.03, 0.10, 0.2);

        let ref_color_pas = Hsv::new(60.0, 0.37, 1.00);
        let tol_color_pas = Hsv::new(0.01, 0.10, 0.2);

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
                    let current_pixel_color = Color::new(px.0[0], px.0[1], px.0[2]);
                    if Self::pixel_matches_hsv(current_pixel_color, ref_color_pas, tol_color_pas) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Passive))));
                    } else if Self::pixel_matches_hsv(
                        current_pixel_color,
                        ref_color_agg,
                        tol_color_agg,
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

    pub fn identify_target_marker(
        &self,
        blank_target: bool,
    ) -> (crate::data::stats_info::TargetMarkerType, Option<Target>) {
        let _timer = Timer::start_new("identify_target_marker");
        let mut coords = Vec::default();

        // Reference color
        let ref_color: Hsv = {
            if !blank_target {
                Hsv::new(355.06848, 0.6293103, 0.9098039)
            } else {
                Hsv::new(240.0, 0.36923078, 0.25490198)
            }
        };

        let default_tol: Hsv = Hsv::new(0.0, 0.0, 0.0);

        let wanted_bounds = self.get_centered_bound_of(0, 0);

        // Collect pixel clouds
        let recv = self.pixel_detection(
            vec![ref_color],
            wanted_bounds.x,
            wanted_bounds.y,
            wanted_bounds.w,
            wanted_bounds.h,
            Some(default_tol),
        );

        // Receive points from channel
        while let Ok(point) = recv.recv() {
            coords.push(point.0);
            #[cfg(debug_assertions)]
            if true {
                Self::debug_color_pixel(
                    "target marker",
                    point.0,
                    point.1,
                    ref_color,
                    default_tol,
                    true,
                );
            }
        }
        // Identify target marker entities
        let target_markers =
            Self::merge_cloud_into_mobs(None, &PointCloud::new(coords), TargetType::TargetMarker);

        // Find biggest target marker
        let res = target_markers.into_iter().max_by_key(|x| x.bounds.size());
        if !blank_target && res.is_none() {
            // Not a red target gotta try white used in farming mode only
            return self.clone().identify_target_marker(true);
        }

        if res.is_some() {
            if blank_target {
                return (TargetMarkerType::Passive, res);
            } else {
                return (TargetMarkerType::Aggressive, res);
            }
        } else {
            return (TargetMarkerType::None, None);
        }
    }
    /// Helps determining the correct searched pixel value
    #[cfg(debug_assertions)]
    pub fn debug_color_pixel(
        label: &str,
        pos: Point,
        current_hsv: Hsv,
        searched_hsv: Hsv,
        tolerance_hsv: Hsv,
        hide_optimised: bool,
    ) {
        let current_hue = current_hsv.hue.to_positive_degrees();
        let current_saturation = current_hsv.saturation;
        let current_value = current_hsv.value;
        let current_obj = (current_hue, current_saturation, current_value);

        let searched_hue = searched_hsv.hue.to_positive_degrees();
        let searched_saturation = searched_hsv.saturation;
        let searched_value = searched_hsv.value;
        let searched_obj = (searched_hue, searched_saturation, searched_value);

        let tol_hue = tolerance_hsv.hue.to_positive_degrees();
        let tol_saturation = tolerance_hsv.saturation;
        let tol_value = tolerance_hsv.value;
        let tol_obj = (tol_hue, tol_saturation, tol_value);

        let optimised_hue = {
            if current_hue > searched_hue {
                current_hue - searched_hue
            } else {
                searched_hue - current_hue
            }
        };

        let optimised_saturation = {
            if current_saturation > searched_saturation {
                current_saturation - searched_saturation
            } else {
                searched_saturation - current_saturation
            }
        };
        let optimised_value = {
            if current_value > searched_value {
                current_value - searched_value
            } else {
                searched_value - current_value
            }
        };
        let optimised_obj = (optimised_hue, optimised_saturation, optimised_value);
        if tol_obj != optimised_obj {
            println!("Found maching pixel for {} at coords X{} Y{} : {:?} reference : {:?} optimised tolerance {:?} current tolerance {:?}", label, pos.x, pos.y, current_obj, searched_obj, optimised_obj,tol_obj)
        } else if !hide_optimised {
            println!("Found maching pixel for {} at coords X{} Y{} : {:?} reference : {:?} already optimised tolerance {:?}", label, pos.x, pos.y, current_obj, searched_obj, tol_obj)
        }
    }

    pub fn get_centered_bound_of(&self, mut width: u32, mut height: u32) -> Bounds {
        let image = self.image.as_ref().unwrap();
        let mid_width = image.width() / 2;
        let mid_height = image.height() / 2;

        if width == 0 {
            width = image.width();
        }
        if height == 0 {
            height = image.height();
        }

        let wanted_width = width / 2;
        let wanted_height = height / 2;

        let min_x = mid_width - wanted_width;
        let min_y = mid_height - wanted_height;
        let max_x = mid_height + wanted_width;
        let max_y = mid_height + wanted_height;
        Bounds {
            x: min_x,
            y: min_y,
            w: max_x,
            h: max_y,
        }
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

        // Calculate 2D euclidian distances to player
        let mut distances = Vec::default();
        for mob in mobs {
            distances.push((mob, mob.get_distance()));
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
