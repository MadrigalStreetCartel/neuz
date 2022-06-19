#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// use image::{codecs::jpeg::JpegEncoder, imageops, ColorType};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use tauri::{Manager, PhysicalPosition, Position};
use rand::prelude::{Rng, SliceRandom};
use serde::{Deserialize, Serialize};

// windows support
use win_screenshot::capture::Image;
use winput::Vk;

mod algo;
mod utils;

use crate::{
    algo::{x_axis_selector, y_axis_selector, AxisClusterComputer, Bounds},
    utils::Timer,
};

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        // .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![start_bot,])
        .run(context)
        .expect("error while running tauri application");
}

enum BotState {
    Idle,
    NoEnemyFound,
    SearchingForEnemy,
    EnemyFound(Mob),
    Attacking(Mob),
    AfterEnemyKill(Mob),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MobType {
    Passive,
    Aggro,
    TargetMarker,
}

#[derive(Debug, Clone, Copy)]
struct Mob {
    mob_type: MobType,
    name_bounds: Bounds,
}

impl Mob {
    pub fn get_attack_coords(&self) -> (u32, u32) {
        let (x, y) = self.name_bounds.get_lowest_center_point();
        (x, y + 20)
    }
}

fn merge_cloud_into_mobs(coords: &[(u32, u32)], mob_type: MobType) -> Vec<Mob> {
    let _timer = Timer::start_new("merge_cloud_into_mobs");

    // Max merge distance
    let max_distance_x: u32 = 20;
    let max_distance_y: u32 = 5;

    // Cluster coordinates in x-direction
    let x_clusters =
        AxisClusterComputer::cluster_by_distance(coords, max_distance_x, x_axis_selector);

    let mut xy_clusters = Vec::default();
    for x_cluster in x_clusters {
        // Cluster current x-cluster coordinates in y-direction
        let local_y_clusters = AxisClusterComputer::cluster_by_distance(
            x_cluster.points_ref(),
            max_distance_y,
            y_axis_selector,
        );
        // Extend final xy-clusters with local y-clusters
        xy_clusters.extend(local_y_clusters.into_iter());
    }

    // Create mobs from clusters
    xy_clusters
        .into_iter()
        .map(|cluster| Mob {
            mob_type,
            name_bounds: cluster.into_approx_rect().into_bounds(),
        })
        .filter(|mob| {
            // Filter out small clusters (likely to cause misclicks)
            mob.name_bounds.size() > (10 * 6)
            // Filter out huge clusters (likely to be Violet Magician Troupe)
            && mob.name_bounds.size() < (220 * 6)
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

fn identify_mobs(image: &Image) -> Vec<Mob> {
    let _timer = Timer::start_new("identify_mobs");

    // Create collections for passive and aggro mobs
    let mut mob_coords_pas: Vec<(u32, u32)> = Vec::default();
    let mut mob_coords_agg: Vec<(u32, u32)> = Vec::default();

    // Reference colors
    let ref_color_pas: [u8; 3] = [0xe8, 0xe8, 0x94]; // Passive mobs
    let ref_color_agg: [u8; 3] = [0xd3, 0x0f, 0x0d]; // Aggro mobs

    // Collect pixel clouds for passive and aggro mobs
    let ignore_area_bottom = 50;
    for (x, y, px) in image.enumerate_pixels() {
        if px.0[3] != 255 || y > image.height() - ignore_area_bottom {
            continue;
        }
        if pixel_matches(&px.0, &ref_color_pas, 2) {
            mob_coords_pas.push((x, y));
        } else if pixel_matches(&px.0, &ref_color_agg, 8) {
            mob_coords_agg.push((x, y));
        }
    }

    // Identify mobs
    let mobs_pas = merge_cloud_into_mobs(&mob_coords_pas, MobType::Passive);
    let mobs_agg = merge_cloud_into_mobs(&mob_coords_agg, MobType::Aggro);

    // Return all mobs
    Vec::from_iter(mobs_pas.into_iter().chain(mobs_agg.into_iter()))
}

fn identify_target_marker(image: &Image) -> Option<Mob> {
    let _timer = Timer::start_new("identify_target_marker");
    let mut coords = Vec::default();

    // Reference color
    let ref_color: [u8; 3] = [246, 90, 106];

    // Collect pixel clouds for target markers
    let ignore_area_bottom = 50;
    for (x, y, px) in image.enumerate_pixels() {
        if px.0[3] != 255 || y > image.height() - ignore_area_bottom {
            continue;
        }
        if pixel_matches(&px.0, &ref_color, 2) {
            coords.push((x, y));
        }
    }

    // Identify target marker entities
    let target_markers = merge_cloud_into_mobs(&coords, MobType::TargetMarker);

    // Find biggest target marker
    target_markers
        .into_iter()
        .max_by_key(|x| x.name_bounds.size())
}

fn find_closest_mob<'a>(image: &Image, mobs: &'a [Mob], avoid_bounds: Option<&Bounds>) -> &'a Mob {
    let _timer = Timer::start_new("find_closest_mob");

    // Calculate middle point of player
    let mid_x = (image.width() / 2) as i32;
    let mid_y = (image.height() / 2) as i32 + 20; // shift mid y point down a bit

    // Calculate 2D euclidian distances to player
    let mut distances = Vec::default();
    for mob in mobs {
        let (x, y) = mob.get_attack_coords();
        let distance =
            (((mid_x - x as i32).pow(2) + (mid_y - y as i32).pow(2)) as f64).sqrt() as i32;
        distances.push((mob, distance));
    }

    // Sort by distance
    distances.sort_by_key(|&(_, distance)| distance);

    if let Some(avoid_bounds) = avoid_bounds {
        // Try finding closest mob that' not the mob to be avoided
        if let Some((mob, _)) = distances
            .iter()
            .filter(|(mob, _)| {
                !avoid_bounds
                    .grow_by(10)
                    .intersects_point(&mob.get_attack_coords())
            })
            .next()
        {
            println!("Found mob avoiding last target.");
            mob
        } else {
            distances.first().unwrap().0
        }
    } else {
        // Return closest mob
        distances.first().unwrap().0
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontendInfo {
    enemy_bounds: Option<Vec<Bounds>>,
    active_enemy_bounds: Option<Bounds>,
    enemy_kill_count: usize,
    is_attacking: bool,
    is_running: bool,
}

impl FrontendInfo {
    fn new() -> Self {
        Self {
            enemy_bounds: None,
            active_enemy_bounds: None,
            enemy_kill_count: 0,
            is_attacking: false,
            is_running: false,
        }
    }

    fn set_enemy_bounds(&mut self, enemy_bounds: Vec<Bounds>) {
        self.enemy_bounds = Some(enemy_bounds);
    }

    fn set_active_enemy_bounds(&mut self, active_enemy_bounds: Bounds) {
        self.active_enemy_bounds = Some(active_enemy_bounds);
    }

    fn set_kill_count(&mut self, enemy_kill_count: usize) {
        self.enemy_kill_count = enemy_kill_count;
    }

    fn set_is_attacking(&mut self, is_attacking: bool) {
        self.is_attacking = is_attacking;
    }

    fn set_is_running(&mut self, is_running: bool) {
        self.is_running = is_running;
    }
}

#[tauri::command]
async fn start_bot(app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();
    let hwnd = window.hwnd().unwrap();

    let mut last_pot_time = Instant::now();
    let mut last_initial_attack_time = Instant::now();
    let mut last_kill_time = Instant::now();
    let mut last_killed_mob_bounds = Bounds::default();
    let mut is_attacking = false;
    let mut kill_count: usize = 0;
    let mut rotation_movement_tries = 0;
    let is_paused = Arc::new(AtomicBool::new(false));

    let local_is_paused = is_paused.clone();
    app_handle.listen_global("toggle_bot", move |_| {
        let was_paused = local_is_paused.load(Ordering::Relaxed);
        local_is_paused.store(!was_paused, Ordering::Relaxed);
    });

    let local_is_paused = is_paused.clone();
    std::thread::spawn(move || {
        let mut state = BotState::SearchingForEnemy;
        let mut rng = rand::thread_rng();

        let mouse = mouse_rs::Mouse::new();

        loop {
            let is_paused = local_is_paused.load(Ordering::Relaxed);

            // Initialize frontend info
            let mut frontend_info = FrontendInfo::new();
            frontend_info.set_is_attacking(is_attacking);
            frontend_info.set_kill_count(kill_count);

            // Update frontend info with running status
            frontend_info.set_is_running(!is_paused);

            // Check whether the bot is paused
            if is_paused {
                app_handle
                    .emit_all("frontend_info", &frontend_info)
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(250));
                continue;
            }

            // Make sure the window is focused
            let focused_hwnd = unsafe { winapi::um::winuser::GetForegroundWindow() };
            if focused_hwnd as isize != hwnd.0 {
                app_handle
                    .emit_all("frontend_info", &frontend_info)
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            // Capture the window
            let image = {
                let _timer = Timer::start_new("capture_window");
                if let Ok(image) = capture_screenshot(hwnd.0) {
                    image
                } else {
                    continue;
                }
            };

            // Consume foodies every 5 seconds and only while attacking
            let current_time = Instant::now();
            let min_pot_time_diff = Duration::from_secs(5);
            if is_attacking
                && current_time.duration_since(last_initial_attack_time) > min_pot_time_diff
                && current_time.duration_since(last_pot_time) > min_pot_time_diff
            {
                winput::send(Vk::_1);
                last_pot_time = current_time;
            }

            // Crop image
            // let (crop_x, crop_y) = (image.width() / 6, image.height() / 6);
            // let image = image.sub_image(crop_x, crop_y, image.width() - crop_x, image.height() - crop_y).to_image();

            match state {
                BotState::Idle => {
                    let total_idle_duration =
                        std::time::Duration::from_millis(rng.gen_range(500..2000));
                    let idle_chunks = rng.gen_range(1..4);
                    let idle_chunk_duration = total_idle_duration / idle_chunks;
                    for _ in 0..idle_chunks {
                        if rng.gen_bool(0.1) {
                            winput::send(Vk::Space);
                        }
                        std::thread::sleep(idle_chunk_duration);
                    }
                    state = BotState::SearchingForEnemy;
                }
                BotState::NoEnemyFound => {
                    // Try rotating first in order to locate nearby enemies
                    if rotation_movement_tries < 10 {
                        // Rotate in random direction for a random duration
                        let key = [Vk::A, Vk::D].choose(&mut rng).unwrap();
                        let rotation_duration =
                            std::time::Duration::from_millis(rng.gen_range(100..500));
                        winput::press(*key);
                        std::thread::sleep(rotation_duration);
                        winput::release(*key);
                        rotation_movement_tries += 1;
                        state = BotState::SearchingForEnemy;
                        continue;
                    }
                    // If rotating multiple times failed, try other movement patterns
                    match rng.gen_range(0..3) {
                        0 => {
                            // Move into a random direction while jumping
                            let key = [Vk::A, Vk::D].choose(&mut rng).unwrap();
                            let rotation_duration =
                                std::time::Duration::from_millis(rng.gen_range(100..350));
                            winput::press(Vk::W);
                            winput::press(Vk::Space);
                            winput::press(*key);
                            std::thread::sleep(rotation_duration);
                            winput::release(*key);
                            winput::release(Vk::Space);
                            winput::release(Vk::W);
                        }
                        1 => {
                            // Move forwards while jumping
                            winput::press(Vk::W);
                            winput::press(Vk::Space);
                            std::thread::sleep(std::time::Duration::from_millis(
                                rng.gen_range(1000..4000),
                            ));
                            winput::release(Vk::Space);
                            winput::release(Vk::W);
                        }
                        2 => {
                            // Move forwards in a slalom pattern
                            let slalom_switch_duration =
                                std::time::Duration::from_millis(rng.gen_range(350..650));
                            let total_slaloms = rng.gen_range(4..8);
                            let mut left = rng.gen_bool(0.5);
                            winput::press(Vk::W);
                            for _ in 0..total_slaloms {
                                winput::press(if left { Vk::A } else { Vk::D });
                                std::thread::sleep(slalom_switch_duration);
                                winput::release(if left { Vk::A } else { Vk::D });
                                left = !left;
                            }
                            winput::release(Vk::W);
                        }
                        _ => unreachable!("Impossible"),
                    }
                    state = BotState::SearchingForEnemy;
                }
                BotState::SearchingForEnemy => {
                    let mobs = identify_mobs(&image);
                    frontend_info.set_enemy_bounds(
                        mobs.iter()
                            .map(|mob| mob.name_bounds.clone())
                            .collect::<Vec<_>>(),
                    );
                    if mobs.is_empty() {
                        // No mobs found, run movement algo
                        state = BotState::NoEnemyFound;
                    } else {
                        // Check if aggro mobs are found first
                        let aggro_mobs = mobs
                            .iter()
                            .filter(|m| m.mob_type == MobType::Aggro)
                            .cloned()
                            .collect::<Vec<_>>();
                        let passive_mobs = mobs
                            .iter()
                            .filter(|m| m.mob_type == MobType::Passive)
                            .cloned()
                            .collect::<Vec<_>>();
                        if !aggro_mobs.is_empty() {
                            println!("Found {} aggro mobs. Those die first.", aggro_mobs.len());
                            let mob = find_closest_mob(&image, aggro_mobs.as_slice(), None);
                            state = BotState::EnemyFound(mob.clone());
                        } else if !passive_mobs.is_empty() {
                            println!("Found {} passive mobs.", passive_mobs.len());
                            let mob = {
                                // Try avoiding detection of last killed mob
                                if Instant::now().duration_since(last_kill_time)
                                    < Duration::from_secs(5)
                                {
                                    println!("Avoiding mob at {:?}", last_killed_mob_bounds);
                                    find_closest_mob(
                                        &image,
                                        passive_mobs.as_slice(),
                                        Some(&last_killed_mob_bounds),
                                    )
                                } else {
                                    find_closest_mob(&image, passive_mobs.as_slice(), None)
                                }
                            };
                            state = BotState::EnemyFound(mob.clone());
                        } else {
                            println!("Mobs were found, but they're neither aggro nor neutral???");
                            state = BotState::NoEnemyFound;
                        }
                    }
                }
                BotState::EnemyFound(mob) => {
                    rotation_movement_tries = 0;

                    // Transform attack coords into local window coords
                    let (x, y) = mob.get_attack_coords();
                    println!(
                        "Trying to attack {} mob at [{},{}]",
                        if mob.mob_type == MobType::Aggro {
                            "aggro"
                        } else {
                            "passive"
                        },
                        x,
                        y
                    );
                    let inner_size = window.inner_size().unwrap();
                    let (x_diff, y_diff) = (
                        image.width() - inner_size.width,
                        image.height() - inner_size.height,
                    );
                    let (window_x, window_y) = (
                        (x.saturating_sub(x_diff / 2)) as i32,
                        (y.saturating_sub(y_diff)) as i32,
                    );
                    let target_cursor_pos = Position::Physical(PhysicalPosition {
                        x: window_x,
                        y: window_y,
                    });

                    // Set cursor position and simulate a click
                    window.set_cursor_position(target_cursor_pos).unwrap();
                    mouse.click(&mouse_rs::types::keys::Keys::LEFT).unwrap();

                    // Wait a few ms before switching state
                    state = BotState::Attacking(mob);
                }
                BotState::Attacking(mob) => {
                    frontend_info.set_active_enemy_bounds(mob.name_bounds);
                    if !is_attacking {
                        last_initial_attack_time = Instant::now();
                        last_pot_time = Instant::now();
                    }
                    if let Some(marker) = identify_target_marker(&image) {
                        // Target marker found
                        is_attacking = true;
                        last_killed_mob_bounds = marker.name_bounds;
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    } else {
                        // Target marker not found
                        if is_attacking {
                            // Enemy was probably killed
                            is_attacking = false;
                            state = BotState::AfterEnemyKill(mob);
                        } else {
                            // Lost target without attacking?
                            state = BotState::SearchingForEnemy;
                        }
                    }
                }
                BotState::AfterEnemyKill(_mob) => {
                    kill_count += 1;
                    last_kill_time = Instant::now();
                    state = {
                        if rng.gen_bool(0.1) {
                            BotState::Idle
                        } else {
                            BotState::SearchingForEnemy
                        }
                    };
                }
            }

            // Convert image to base64 and send to frontend
            // let mut buf = Vec::new();
            // let downscaled_image = imageops::resize(
            //     &image,
            //     image.width() / 8,
            //     image.height() / 8,
            //     imageops::Nearest,
            // );
            // let mut encoder = JpegEncoder::new_with_quality(&mut buf, 90);
            // encoder
            //     .encode(
            //         &downscaled_image,
            //         downscaled_image.width(),
            //         downscaled_image.height(),
            //         ColorType::Rgba8,
            //     )
            //     .unwrap();
            // let base64 = format!("data:image/jpeg;base64,{}", base64::encode(&buf));
            // app_handle
            //     .emit_all(
            //         "bot_visualizer_update",
            //         [
            //             base64,
            //             image.width().to_string(),
            //             image.height().to_string(),
            //         ],
            //     )
            //     .unwrap();

            // Update frontend info and send it over
            frontend_info.set_kill_count(kill_count);
            app_handle
                .emit_all("frontend_info", &frontend_info)
                .unwrap();
        }
    });
}

#[cfg(target_os = "windows")]
fn capture_screenshot(hwnd: isize) -> Result<Image, ()> {
    use win_screenshot::capture::capture_window;

    capture_window(hwnd).map_err(|_| ())
}

#[cfg(target_os = "linux")]
fn capture_screenshot(window: &Window) {
    unimplemented!()
}
