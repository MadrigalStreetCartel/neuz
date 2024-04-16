use std::{ sync::mpsc::sync_channel, time::Instant };
use std::collections::HashMap;
//use libscreenshot::shared::Area;
use libscreenshot::{ ImageBuffer, WindowCaptureProvider };
use rayon::iter::{
    IntoParallelIterator,
    IntoParallelRefIterator,
    ParallelBridge,
    ParallelIterator,
};
use slog::Logger;
use tauri::Window;

use crate::{
    data::{
        Bounds,
        ClientStats,
        CloudDetection,
        CloudDetectionConfig,
        CloudDetectionKind,
        CloudDetectionCategorie,
        Point,
        PointCloud,
        Target,
    },
    platform::{ IGNORE_AREA_BOTTOM, IGNORE_AREA_TOP },
    utils::Timer,
};

#[derive(Debug, Clone)]
pub struct ImageAnalyzer<'a> {
    image: Option<ImageBuffer>,
    pub window_id: u64,
    pub client_stats: ClientStats,
    _logger: &'a Logger,
}

impl<'a> ImageAnalyzer<'a> {
    pub fn new(window: &Window, logger: &'a Logger) -> Self {
        Self {
            _logger: logger,
            window_id: 0,
            image: None,
            client_stats: ClientStats::new(window.to_owned(), logger),
        }
    }

    pub fn image_is_some(&self) -> bool {
        self.image.is_some()
    }

    pub fn capture_window(&mut self, logger: &Logger) {
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

    pub fn pixel_detection(&mut self, config: &Vec<CloudDetectionConfig>) -> Vec<CloudDetection> {
        const DETECTION_BUFFER: usize = 4096;
        let (snd, recv) = sync_channel::<(CloudDetectionCategorie, Point)>(DETECTION_BUFFER);
        let image = self.image.as_ref().unwrap();

        let config = &config
            .iter()
            .filter(|x| x.enabled)
            .collect::<Vec<_>>();

        let mut max_x = config
            .iter()
            .map(|x| x.bounds.w)
            .max()
            .unwrap_or(0);
        let mut max_y = config
            .iter()
            .map(|x| x.bounds.h)
            .max()
            .unwrap_or(0);

        let min_x = config
            .iter()
            .map(|x| x.bounds.x)
            .min()
            .unwrap_or(0);
        let min_y = config
            .iter()
            .map(|x| x.bounds.y)
            .min()
            .unwrap_or(0);

        let image_width = image.width();
        if max_x == 0 {
            max_x = image_width;
        }

        let image_height = image.height();
        if max_y == 0 {
            max_y = image_height;
        }
        let image_height = image_height.checked_sub(IGNORE_AREA_BOTTOM).unwrap_or(image_height);

        image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                // Skip this row if it's in an ignored area
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if
                    y <= IGNORE_AREA_TOP ||
                    y > image_height ||
                    y > IGNORE_AREA_TOP + max_y ||
                    y > max_y ||
                    y < min_y
                {
                    //println!("returning early on y: {}", y);
                    return;
                }

                // Loop over columns
                'outer: for (x, _, px) in row {
                    if px.0[3] != 255 || x >= max_x {
                        return;
                    } else if x < min_x {
                        continue;
                    }

                    let matched_pixel = config
                        .iter()
                        .find(|config| {
                            config.is_within_bounds(x, y) && config.pixel_compare(&px.0)
                        });

                    if let Some(config) = matched_pixel {
                        #[allow(dropping_copy_types)]
                        drop(
                            snd.try_send((config.config.clone(), Point::new(x, y))).map_err(|err| {
                                eprintln!("Error sending data: {}", err);
                            })
                        );
                        // Continue to next column
                        continue 'outer;
                    }
                }
            });

        // Receive points from channel
        let clouds = {
            let mut clouds: HashMap<CloudDetectionCategorie, PointCloud> = HashMap::default();
            for i in 0..config.len() {
                clouds.insert(config[i].config, PointCloud::default());
            }
            while let Ok((config, point)) = recv.recv() {
                if let Some(cloud) = clouds.get_mut(&config) {
                    cloud.push(point);
                }
            }
            clouds
        };

        let mut detected_clouds = clouds
            .par_iter()
            .map(move |(detector, cloud)| {
                CloudDetection {
                    kind: *detector,
                    cloud: cloud.clone(),
                }
            })
            .collect();

        self.client_stats.update(&detected_clouds);

        let result = detected_clouds.par_iter().find_map_first(move |cloud| {
            match cloud.kind {
                CloudDetectionCategorie::Mover(t) => {
                    match t {
                        CloudDetectionKind::Target(is_red) => {
                            let target = cloud.process_target();
                            if target.is_some() {
                                return Some((target, is_red));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            None
        });

        let target = if let Some((target, _is_red)) = result {
            self.client_stats.target_distance = Some(
                self.get_target_distance_to_player(target.unwrap())
            );
            target
        } else {
            self.client_stats.target_distance = None;

            None
        };

        self.client_stats.target_marker = target;
        self.client_stats.target_on_screen = target.is_some();

        // remove used clouds
        detected_clouds.retain(|x| {
            match x.kind {
                CloudDetectionCategorie::Mover(t) => {
                    match t {
                        CloudDetectionKind::Target(_) => false,
                        _ => true,
                    }
                }
                CloudDetectionCategorie::Stat(_) => false,
                _ => true,
            }
        });

        detected_clouds

        // Set clouds
    }

    pub fn identify_mobs(&self, clouds: &Vec<CloudDetection>) -> Vec<Target> {
        let mut result: Vec<Target> = Vec::default();
        for cloud in clouds {
            match cloud.kind {
                CloudDetectionCategorie::Mover(t) => {
                    match t {
                        CloudDetectionKind::Mob(_) => {
                            let mobs = cloud.process_mobs();
                            result.extend(mobs.into_iter());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        result
    }

    pub fn get_target_distance_to_player(&self, target: Target) -> i32 {
        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = image.width() / 2;
        let mid_y = image.height() / 2;

        let point = Point::new(mid_x, mid_y);

        target.get_target_distance_to(point)
    }

    /// Distance: `[0..=500]`
    pub fn find_closest_mob<'b>(
        &self,
        mobs: &'a [Target],
        avoid_list: Option<&Vec<(Bounds, Instant, u128)>>,
        max_distance: i32,
        _logger: &Logger
    ) -> Option<&'a Target> {
        let _timer = Timer::start_new("find_closest_mob");

        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = image.width() / 2;
        let mid_y = image.height() / 2;
        let point = Point::new(mid_x, mid_y);

        // Calculate 2D euclidian distances to player
        let mut distances: Vec<(&Target, i32)> = mobs
            .into_par_iter()
            .map(|mob| {
                let distance = mob.get_target_distance_to(point);
                (mob, distance)
            })
            .collect();

        // Sort by distance
        distances.sort_by_key(|&(_, distance)| distance);

        // Remove mobs that are too far away
        distances = distances
            .into_par_iter()
            .filter(|&(_, distance)| distance <= max_distance)
            .collect();

        if let Some(avoided_bounds) = avoid_list {
            // Try finding closest mob that's not the mob to be avoided
            let closest = distances.par_iter().find_first(|(mob, _distance)| {
                //*distance > 55
                let coords = mob.get_attack_coords();
                let avoided_bounds = avoided_bounds
                    .par_iter().find_first(|avoided_item| { !avoided_item.0.contains_point(&coords) });

                if let Some(_) = avoided_bounds {
                    true
                } else {
                    false
                }
            });

            if let Some((mob, _distance)) = closest {
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
