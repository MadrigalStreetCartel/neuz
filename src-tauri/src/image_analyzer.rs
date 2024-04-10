use std::{ sync::mpsc::sync_channel, time::Instant };
use std::collections::HashMap;
//use libscreenshot::shared::Area;
use libscreenshot::{ ImageBuffer, WindowCaptureProvider };
use rayon::iter::{ ParallelBridge, ParallelIterator };
use slog::Logger;
use tauri::Window;

use crate::{
    data::{
        point_selector,
        AliveState,
        Bounds,
        ClientStats,
        PixelCloud,
        PixelCloudConfig,
        PixelCloudKind,
        PixelCloudKindCategorie,
        Point,
        PointCloud,
        Target,
        TargetType,
    },
    ipc::FarmingConfig,
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

    pub fn pixel_detection(&mut self, config: &Vec<PixelCloudConfig>) -> Vec<PixelCloud> {
        const DETECTION_BUFFER: usize = 4096;
        let (snd, recv) = sync_channel::<(PixelCloudKindCategorie, Point)>(DETECTION_BUFFER);
        let image = self.image.as_ref().unwrap();
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

        if max_x == 0 {
            max_x = image.width();
        }

        if max_y == 0 {
            max_y = image.height();
        }

        let image_height = image.height();
        image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                // Skip this row if it's in an ignored area
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if
                    y <= IGNORE_AREA_TOP ||
                    y > image_height.checked_sub(IGNORE_AREA_BOTTOM).unwrap_or(image_height) ||
                    y > IGNORE_AREA_TOP + max_y ||
                    y > max_y ||
                    y < min_y
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
                    for config in config.iter() {
                        // Check if the pixel matches any of the reference colors
                        if config.is_within_bounds(x, y) && config.pixel_compare(&px.0) {
                            #[allow(dropping_copy_types)]
                            drop(
                                snd
                                    .try_send((config.detector.clone(), Point::new(x, y)))
                                    .map_err(|err| {
                                        eprintln!("Error sending data: {}", err);
                                    })
                            );
                            // Continue to next column
                            continue 'outer;
                        }
                    }
                }
            });

        // Receive points from channel
        let clouds = {
            let mut clouds: HashMap<PixelCloudKindCategorie, PointCloud> = HashMap::default();
            for i in 0..config.len() {
                clouds.insert(config[i].detector, PointCloud::default());
            }
            while let Ok((config, point)) = recv.recv() {
                if let Some(cloud) = clouds.get_mut(&config) {
                    cloud.push(point);
                } else {
                    let mut cloud = PointCloud::default();
                    cloud.push(point);
                    clouds.insert(config, cloud);
                }
            }
            clouds
        };

        let mut pixel_clouds = {
            let mut pixel_clouds: Vec<PixelCloud> = Vec::default();
            for (detector, cloud) in clouds {
                pixel_clouds.push(PixelCloud {
                    kind: detector,
                    cloud,
                });
            }
            pixel_clouds
        };
        self.client_stats.update_v2(&pixel_clouds);
        self.client_stats.has_tray_open = self.client_stats.detect_stat_tray();
        self.client_stats.is_alive = {
            if !self.client_stats.has_tray_open {
                AliveState::StatsTrayClosed
            } else if self.client_stats.hp.value > 0 {
                AliveState::Alive
            } else {
                AliveState::Dead
            }
        };
        self.client_stats.target_is_npc =
            self.client_stats.target_hp.value == 100 && self.client_stats.target_mp.value == 0;
        self.client_stats.target_is_mover = self.client_stats.target_mp.value > 0;
        self.client_stats.target_is_alive = self.client_stats.target_hp.value > 0;
        let mut _is_red_target = false;
        let target: Option<Target> = {
            let mut result = None;
            for cloud in &pixel_clouds {
                if result.is_some() {
                    continue;
                }
                match cloud.kind {
                    PixelCloudKindCategorie::Mover(t) => {
                        match t {
                            PixelCloudKind::Target(is_red) => {
                                let target = cloud.process_target();
                                if result.is_none() && target.is_some() {
                                    if is_red {
                                        _is_red_target = true;
                                    }
                                    result = target;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            result
        };
        self.client_stats.target_marker = target;
        self.client_stats.target_on_screen = target.is_some();

        if self.client_stats.target_on_screen {
            self.client_stats.target_distance = Some(
                self.get_target_marker_distance(target.unwrap())
            );
        } else {
            self.client_stats.target_distance = None;
        }

        // remove used clouds
        pixel_clouds.retain(|x| {
            match x.kind {
                PixelCloudKindCategorie::Mover(t) => {
                    match t {
                        PixelCloudKind::Target(_) => false,
                        _ => true,
                    }
                }
                PixelCloudKindCategorie::Stat(_) => false,
                _ => true,
            }
        });

        pixel_clouds

        // Set clouds
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

    pub fn identify_mobs(&self, clouds: &Vec<PixelCloud>) -> Vec<Target> {
        let mut result: Vec<Target> = Vec::default();
        for cloud in clouds {
            match cloud.kind {
                PixelCloudKindCategorie::Mover(t) => {
                    match t {
                        PixelCloudKind::Mob(_) => {
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

    pub fn get_target_marker_distance(&self, target: Target) -> i32 {
        let image = self.image.as_ref().unwrap();

        // Calculate middle point of player
        let mid_x = (image.width() / 2) as i32;
        let mid_y = (image.height() / 2) as i32;

        // Calculate 2D euclidian distances to player
        let point = target.bounds.get_lowest_center_point();

        (
            ((mid_x - (point.x as i32)).pow(2) + (mid_y - (point.y as i32)).pow(2)) as f64
        ).sqrt() as i32
    }
    /// Distance: `[0..=500]`
    pub fn find_closest_mob<'b>(
        &self,
        mobs: &'a [Target],
        //avoid_bounds: Option<&Bounds>,
        avoid_list: Option<&Vec<(Bounds, Instant, u128)>>,
        max_distance: i32,
        _logger: &Logger
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
            let distance = (
                ((mid_x - (point.x as i32)).pow(2) + (mid_y - (point.y as i32)).pow(2)) as f64
            ).sqrt() as i32;
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
            if
                let Some((mob, _distance)) = distances.iter().find(|(mob, _distance)| {
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
                })
            {
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
