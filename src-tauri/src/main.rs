#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use algo::Bounds;
use image::codecs::jpeg::JpegEncoder;
use image::imageops;
use image::ColorType;
use rand::Rng;
use rand::prelude::SliceRandom;
use tauri::Manager;

mod algo;

use tauri::PhysicalPosition;
use tauri::Position;
use win_screenshot::capture::Image;
use winput::Vk;

use crate::algo::x_axis_selector;
use crate::algo::y_axis_selector;
use crate::algo::AxisClusterComputer;

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
    AfterEnemyKill,
    SearchingForEnemy,
    EnemyFound(Mob),
    Attacking(Mob),
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
    target_markers.into_iter().max_by_key(|x| x.name_bounds.size())
}

fn find_closest_mob<'a>(image: &Image, mobs: &'a [Mob]) -> &'a Mob {
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

    // Return closest mob
    distances
        .iter()
        .min_by_key(|&(_, distance)| distance)
        .unwrap()
        .0
}

#[tauri::command]
async fn start_bot(app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();
    let hwnd = window.hwnd().unwrap();

    std::thread::spawn(move || {
        let mut state = BotState::SearchingForEnemy;
        let mut rng = rand::thread_rng();
        let mouse = mouse_rs::Mouse::new();

        loop {
            // Make sure that window is focused
            let focused_hwnd = unsafe { winapi::um::winuser::GetForegroundWindow() };
            if focused_hwnd as isize != hwnd.0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            // Capture the window
            let image = {
                if let Ok(image) = capture_screenshot(hwnd.0) {
                    image
                } else {
                    continue;
                }
            };

            // Crop image
            // let (crop_x, crop_y) = (image.width() / 6, image.height() / 6);
            // let image = image.sub_image(crop_x, crop_y, image.width() - crop_x, image.height() - crop_y).to_image();

            match state {
                BotState::Idle => {
                    let total_idle_duration = std::time::Duration::from_millis(rng.gen_range(250..2000));
                    let idle_chunks = rng.gen_range(1..4);
                    let idle_chunk_duration = total_idle_duration / idle_chunks;
                    for _ in 0..idle_chunks {
                        if rng.gen_bool(0.1) {
                            winput::press(Vk::Space);
                        }
                        std::thread::sleep(idle_chunk_duration);
                    }
                    state = BotState::SearchingForEnemy;
                }
                BotState::AfterEnemyKill => {
                    if rng.gen_bool(0.05) {
                        state = BotState::Idle;
                    } else {
                        state = BotState::SearchingForEnemy;
                    }
                },
                BotState::NoEnemyFound => {
                    match rng.gen_range(0..3) {
                        0 => {
                            // Rotate in random direction for a random duration
                            let key = [Vk::A, Vk::D].choose(&mut rng).unwrap();
                            let rotation_duration = std::time::Duration::from_millis(rng.gen_range(250..750));
                            winput::press(*key);
                            std::thread::sleep(rotation_duration);
                            winput::release(*key);
                        },
                        1 => {
                            // Move in random direction for a random duration
                            let key = [Vk::A, Vk::D].choose(&mut rng).unwrap();
                            let rotation_duration = std::time::Duration::from_millis(rng.gen_range(0..500));
                            let movement_duration = std::time::Duration::from_millis(rng.gen_range(250..750));
                            winput::press(Vk::W);
                            winput::press(*key);
                            std::thread::sleep(rotation_duration);
                            winput::release(*key);
                            std::thread::sleep(movement_duration);
                            winput::release(Vk::W);
                        },
                        2 => {
                            // Move forwards and jump at random intervals
                            let jumps = rng.gen_range(0..3);
                            winput::press(Vk::W);
                            for _ in 0..jumps {
                                winput::send(Vk::Space);
                                std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(1300..2000)));
                            }
                            winput::release(Vk::W);
                        },
                        _ => unreachable!("IMPOSSIBRU")
                    }
                    state = BotState::SearchingForEnemy;
                }
                BotState::SearchingForEnemy => {
                    let mobs = identify_mobs(&image);
                    app_handle
                        .emit_all(
                            "bot_visualizer_enemy_bounds",
                            mobs.iter()
                                .map(|mob| mob.name_bounds.clone())
                                .collect::<Vec<_>>(),
                        )
                        .unwrap();
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
                            println!(
                                "Found {} aggro mobs.",
                                aggro_mobs.len()
                            );
                            let mob = find_closest_mob(&image, aggro_mobs.as_slice());
                            state = BotState::EnemyFound(mob.clone());
                        } else if !passive_mobs.is_empty() {
                            println!("Found {} passive mobs.", passive_mobs.len());
                            let mob = find_closest_mob(&image, passive_mobs.as_slice());
                            state = BotState::EnemyFound(mob.clone());
                        } else {
                            println!("Mobs were found, but they're neither aggro nor passive.");
                            state = BotState::NoEnemyFound;
                        }
                    }
                }
                BotState::EnemyFound(mob) => {
                    // Transform attack coords into local window coords
                    let (x, y) = mob.get_attack_coords();
                    println!("Trying to attack {} mob at [{},{}]", if mob.mob_type == MobType::Aggro { "aggro" } else { "passive" }, x, y);
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
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    state = BotState::Attacking(mob);
                }
                BotState::Attacking(_mob) => {
                    if let Some(_) = identify_target_marker(&image) {
                        // Target marker found, do nothing
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    } else  {
                        // Target marker not found, return to search state
                        state = BotState::AfterEnemyKill;
                    }
                }
            }

            // Convert image to base64 and send to frontend
            let mut buf = Vec::new();
            let downscaled_image = imageops::resize(
                &image,
                image.width() / 8,
                image.height() / 8,
                imageops::Nearest,
            );
            let mut encoder = JpegEncoder::new_with_quality(&mut buf, 90);
            encoder
                .encode(
                    &downscaled_image,
                    downscaled_image.width(),
                    downscaled_image.height(),
                    ColorType::Rgba8,
                )
                .unwrap();
            let base64 = format!("data:image/jpeg;base64,{}", base64::encode(&buf));
            app_handle
                .emit_all(
                    "bot_visualizer_update",
                    [
                        base64,
                        image.width().to_string(),
                        image.height().to_string(),
                    ],
                )
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
