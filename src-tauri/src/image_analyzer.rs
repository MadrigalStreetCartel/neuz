use image::{Rgb, Rgba};
use palette::{convert::IntoColorUnclamped, FromColor, Hsv, IntoColor};
use std::{
    sync::mpsc::{sync_channel, Receiver},
    time::Instant,
};

//use libscreenshot::shared::Area;
use libscreenshot::{ImageBuffer, WindowCaptureProvider};
use rayon::iter::{ParallelBridge, ParallelIterator};
use slog::Logger;
use tauri::Window;

use crate::{
    data::{point_selector, Bounds, ClientStats, MobType, Point, PointCloud, Target, TargetType},
    ipc::FarmingConfig,
    platform::{eval_draw_bounds, eval_shown_debug_overlay, IGNORE_AREA_BOTTOM, IGNORE_AREA_TOP},
    utils::Timer,
};
type HsvTolerance = [f32; 3];

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
    window: Window,
    logger: Logger,

    // frame counting
    frame_count: u64,
    last_frame_time: Instant,
}

impl ImageAnalyzer {
    pub fn new(window: &Window, logger: &Logger) -> Self {
        let mut res = Self {
            window_id: 0,
            image: None,
            client_stats: ClientStats::new(window.to_owned()),
            window: window.to_owned(),
            logger: logger.clone(),

            frame_count: 0,
            last_frame_time: Instant::now(),
        };
        res
    }

    pub fn image_is_some(&self) -> bool {
        self.image.is_some()
    }

    pub fn capture_window(&mut self, logger: &Logger) {
        let _timer = Timer::start_new("capture_window");
        if self.window_id == 0 {
            return;
        }
        // calculate fps
        let now = Instant::now();
        let elapsed = now - self.last_frame_time;
        if elapsed.as_secs() >= 1 {
            let fps = self.frame_count;
            self.frame_count = 0;
            self.last_frame_time = now;
            self.window
                .eval(&format!("debugOverlay.fpsElement.setFps({});", fps))
                .unwrap();
        } else {
            self.frame_count += 1;
        }

        eval_shown_debug_overlay(&self.window, false);
        if let Some(provider) = libscreenshot::get_window_capture_provider() {
            if let Ok(image) = provider.capture_window(self.window_id) {
                eval_shown_debug_overlay(&self.window, true);
                self.frame_count += 1;
                self.image = Some(image);
            } else {
                slog::warn!(logger, "Failed to capture window"; "window_id" => self.window_id);
            }
        }
    }

    pub fn image_process<F>(&self, bounds: Vec<(Bounds, Option<F>)>)
    where
        F: Fn(u32, u32, &[u8; 4]) -> () + Sync + Send,
    {
        let image = self.image.as_ref().unwrap();
        if bounds.is_empty() {
            return;
        }
        for (bound, callback) in bounds {
            if let None = callback {
                // TODO use a guard
                continue;
            }
            let [min_x, min_y, max_x, max_y] = bound.to_detection_area();
            let safe_height = image
                .height()
                .checked_sub(IGNORE_AREA_BOTTOM)
                .unwrap_or(max_x);
            image.enumerate_rows().par_bridge().for_each(|(y, row)| {
                if y <= IGNORE_AREA_TOP
                    || y > safe_height
                    || y > IGNORE_AREA_TOP + max_y
                    || y > max_y
                    || y < min_y
                {
                    return;
                }
                'outer: for (x, _, px) in row {
                    let px = px as &Rgba<u8>;
                    if px.0[3] != 255 || x >= max_x {
                        return;
                    } else if x < min_x {
                        continue 'outer;
                    }
                    callback.as_ref().expect("having a guard is cool")(x, y, &px.0);
                }
            });
        }
    }

    pub fn pixel_detection(
        &self,
        colors: &[(
            Bounds,
            Hsv,
            HsvTolerance,
            Box<dyn Fn(u32, u32) + Sync + Send>,
        )],
    ) {
        self.image_process(
            colors
                .iter()
                .map(|(b, ref_color, tolerance, cb)| {
                    (
                        *b,
                        Some(Box::new(move |x, y, pixel: &[u8; 4]| {
                            let pixel_hsv = palette::Srgb::new(pixel[0], pixel[1], pixel[2]);
                            let pixel_hsv = Hsv::from_color(pixel_hsv.into_format());
                            if Self::pixel_matches(&pixel_hsv, ref_color, *tolerance) {
                                cb(x, y);
                            }
                        })),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    fn merge_cloud_into_mobs(
        window: &Window,
        config: Option<&FarmingConfig>,
        cloud: &PointCloud,
        mob_type: TargetType, //ignore_size: bool,
    ) -> Vec<Target> {
        let _timer = Timer::start_new("merge_cloud_into_mobs");

        // Max merge distance
        let max_distance_x: u32 = 10;
        let max_distance_y: u32 = 7;

        let mut min_w = 10;
        let min_h = 8;
        let mut max_w = 180;
        let max_h = 13;

        if let Some(config) = config {
            min_w = config.min_mobs_name_width();
            max_w = config.max_mobs_name_width();
        }

        // Cluster coordinates in x-y directions
        let xy_clusters = cloud.cluster_by_distance_2d(max_distance_x, max_distance_y);

        // Create mobs from clusters
        xy_clusters
            .into_iter()
            .map(|cluster| Target {
                target_type: mob_type,
                bounds: cluster.to_bounds(),
            })
            .filter(|mob| {
                let is_valid = mob.bounds.w >= min_w
                    && mob.bounds.h >= min_h
                    && mob.bounds.w <= max_w
                    && mob.bounds.h <= max_h;
                if mob_type != TargetType::TargetMarker && is_valid {
                    eval_draw_bounds(window, false, mob.bounds);
                }
                if let Some(config) = config {
                    // Filter targets

                    is_valid
                } else {
                    true
                }
            })
            .collect()
    }

    #[inline(always)]
    pub fn pixel_matches(color_ref: &Hsv, color_pixel: &Hsv, tolerance: HsvTolerance) -> bool {
        let h_dif =
            (color_ref.hue.into_positive_degrees() - color_pixel.hue.into_positive_degrees()).abs();
        if h_dif > tolerance[0] {
            return false;
        }
        let s_dif = (color_ref.saturation - color_pixel.saturation).abs();
        if s_dif > tolerance[1] {
            return false;
        }
        let v_dif = (color_ref.value - color_pixel.value).abs();
        if v_dif > tolerance[2] {
            return false;
        }
        true
    }

    pub fn identify_mobs(&self, config: &FarmingConfig) -> Vec<Target> {
        let _timer = Timer::start_new("identify_mobs");

        // Create collections for passive and aggro mobs
        let mut mob_coords_pas: Vec<Point> = Vec::default();
        let mut mob_coords_agg: Vec<Point> = Vec::default();

        // Collect pixel clouds
        struct MobPixel(u32, u32, TargetType);
        let (snd, recv) = sync_channel::<MobPixel>(4096);
        let snd_clone = snd.clone();
        let bounds = Bounds::new(
            0,
            0,
            self.image.as_ref().unwrap().width(),
            self.image.as_ref().unwrap().height(),
        );
        self.pixel_detection(&[
            (
                bounds,
                Hsv::new(60.0, 0.36, 0.91),
                [0.0, 0.2, 0.08],
                Box::new(move |x, y| {
                    #[allow(dropping_copy_types)]
                    drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Passive))));
                }),
            ),
            (
                bounds,
                Hsv::new(0.0, 0.98, 0.918),
                [0.1, 0.1, 0.2],
                Box::new(move |x, y| {
                    #[allow(dropping_copy_types)]
                    drop(snd_clone.send(MobPixel(x, y, TargetType::Mob(MobType::Aggressive))));
                }),
            ),
        ]);
        while let Ok(px) = recv.recv() {
            match px.2 {
                TargetType::Mob(MobType::Passive) => mob_coords_pas.push(Point::new(px.0, px.1)),
                TargetType::Mob(MobType::Aggressive) => mob_coords_agg.push(Point::new(px.0, px.1)),
                _ => unreachable!(),
            }
        }

        // Categorize mobs
        let mobs_pas = Self::merge_cloud_into_mobs(
            &self.window,
            Some(config),
            &PointCloud::new(mob_coords_pas),
            TargetType::Mob(MobType::Passive),
        );
        let mobs_agg = Self::merge_cloud_into_mobs(
            &self.window,
            Some(config),
            &PointCloud::new(mob_coords_agg),
            TargetType::Mob(MobType::Aggressive),
        );

        // Return all mobs
        Vec::from_iter(mobs_agg.into_iter().chain(mobs_pas))
    }
    pub fn identify_target_marker(&self, blue_target: bool) -> Option<Target> {
        let _timer = Timer::start_new("identify_target_marker");
        let mut coords = Vec::default();

        // Reference color
        let ref_color: (Hsv, &HsvTolerance) = {
            if blue_target {
                (Hsv::new(240.0, 0.37, 0.25), &[0.0, 0.01, 0.01]) //A4B4E1 -- blueish - more center of the arrow
            } else {
                (Hsv::new(4.5, 0.68, 0.61), &[0.02, 0.01, 0.01]) //F65A6A -- redish
            }
        };

        let (snd, recv) = sync_channel::<Point>(4096);
        let bounds = Bounds::new(
            0,
            0,
            self.image.as_ref().unwrap().width(),
            self.image.as_ref().unwrap().height(),
        );
        self.image_process(vec![(
            bounds,
            Some(move |x, y, pixel: &[u8; 4]| {
                if Self::pixel_matches(
                    &Hsv::from_color(
                        palette::Srgb::new(pixel[0], pixel[1], pixel[2]).into_format(),
                    ),
                    &ref_color.0,
                    *ref_color.1,
                ) {
                    #[allow(dropping_copy_types)]
                    drop(snd.send(Point::new(x, y)));
                }
            }),
        )]);

        // // Receive points from channel
        while let Ok(point) = recv.recv() {
            coords.push(point);
        }

        // Identify target marker entities
        let target_markers = Self::merge_cloud_into_mobs(
            &self.window,
            None,
            &PointCloud::new(coords),
            TargetType::TargetMarker,
        );

        if !blue_target && target_markers.is_empty() {
            return self.identify_target_marker(true);
        }

        // Find biggest target marker
        target_markers.into_iter().max_by_key(|x| x.bounds.size())
    }

    pub fn get_target_marker_distance(&self, target: Target) -> i32 {
        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = (image.width() / 2) as i32;
        let mid_y = (image.height() / 2) as i32;

        // Calculate 2D euclidian distances to player
        let point = target.bounds.get_lowest_center_point();

        (((mid_x - (point.x as i32)).pow(2) + (mid_y - (point.y as i32)).pow(2)) as f64).sqrt()
            as i32
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
            let distance = (((mid_x - (point.x as i32)).pow(2) + (mid_y - (point.y as i32)).pow(2))
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
