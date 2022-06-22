#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// use image::{codecs::jpeg::JpegEncoder, imageops, ColorType};

use std::{
    sync::{mpsc::sync_channel, Arc},
    time::{Duration, Instant},
};

use parking_lot::RwLock;
use rand::{prelude::{Rng, SliceRandom}, seq::index};
use rayon::prelude::*;
use tauri::{Manager, PhysicalPosition, Position, Window};

// windows support
#[cfg(target_os = "windows")]
use win_screenshot::capture::Image;
#[cfg(target_os = "windows")]
use winput::Vk;

// linux support
#[cfg(target_os = "linux")]
use tfc::{Context, Error, traits::*};


mod algo;
mod ipc;
mod utils;

use crate::{
    algo::{x_axis_selector, y_axis_selector, AxisClusterComputer, Bounds},
    ipc::{BotConfig, FrontendInfo, SlotType},
    utils::Timer,
};

type Image = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

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
    Interrupted,
    NoEnemyFound,
    SearchingForEnemy,
    EnemyFound(Mob),
    Attacking(Mob),
    AfterEnemyKill(Mob),
}

enum Keymode {
    Press,
    Hold,
    Release,
}

#[derive(Debug, Clone, Copy)]
enum Key {
    // 0-9
    N,
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
    VIII,
    IX,
    X,
    // WASD
    W,
    A,
    S,
    D,
    Space,
}

impl From<usize> for Key {
    fn from(index: usize) -> Self {
        use Key::*;
        match index {
            0 => N,
            1 => I,
            2 => II,
            3 => III,
            4 => IV,
            5 => V,
            6 => VI,
            7 => VII,
            8 => VIII,
            9 => IX,
            _ => unreachable!("Invalid Index (expected 0-9)")
        }
    }
}

#[cfg(target_os="windows")]
impl Into<Vk> for Keys {
  fn into(self) -> Vk {
    use Key::*;
    match index {
        N => Vk::_0,
        I => Vk::_1,
        II => Vk::_2,
        III => Vk::_3,
        IV => Vk::_4,
        V => Vk::_5,
        VI => Vk::_6,
        VII => Vk::_7,
        VIII => Vk::_8,
        IX => Vk::_9,
        W => Vk::W,
        A => Vk::A,
        S => Vk::S,
        D => Vk::D,
        Space => Vk::Space,
        _ => unreachable!("Invalid food index"),
    }
  }
}

#[cfg(target_os="linux")]
impl Into<char> for Key {
    fn into(self) -> char {
    use Key::*;
      match self {
        N => '0',
        I => '1',
        II => '2',
        III => '3',
        IV => '4',
        V => '5',
        VI => '6',
        VII => '7',
        VIII => '8',
        IX => '9',
        W => 'w',
        A => 'a',
        S => 's',
        D => 'd',
        Space => ' ', // TODO
        _ => unreachable!("Invalid food index"),
      }
    }
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
        (x, y + 25)
    }
}

fn merge_cloud_into_mobs(coords: &[(u32, u32)], mob_type: MobType) -> Vec<Mob> {
    let _timer = Timer::start_new("merge_cloud_into_mobs");

    // Max merge distance
    let max_distance_x: u32 = 24;
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

    // Collect pixel clouds
    struct MobPixel(u32, u32, MobType);
    let ignore_area_bottom = 90;
    let (snd, recv) = sync_channel::<MobPixel>(4096);
    image
        .enumerate_rows()
        .par_bridge()
        .for_each(move |(y, row)| {
            if y > image.height() - ignore_area_bottom {
                return;
            }
            for (x, _, px) in row {
                if px.0[3] != 255 || y > image.height() - ignore_area_bottom {
                    return;
                }
                if pixel_matches(&px.0, &ref_color_pas, 2) {
                    snd.send(MobPixel(x, y, MobType::Passive)).unwrap();
                } else if pixel_matches(&px.0, &ref_color_agg, 8) {
                    snd.send(MobPixel(x, y, MobType::Aggro)).unwrap();
                }
            }
        });
    while let Ok(px) = recv.recv() {
        match px.2 {
            MobType::Passive => mob_coords_pas.push((px.0, px.1)),
            MobType::Aggro => mob_coords_agg.push((px.0, px.1)),
            _ => unreachable!(),
        }
    }

    // Identify mobs
    let mobs_pas = merge_cloud_into_mobs(&mob_coords_pas, MobType::Passive);
    let mobs_agg = merge_cloud_into_mobs(&mob_coords_agg, MobType::Aggro);

    // Return all mobs
    Vec::from_iter(mobs_agg.into_iter().chain(mobs_pas.into_iter()))
}

fn identify_target_marker(image: &Image) -> Option<Mob> {
    let _timer = Timer::start_new("identify_target_marker");
    let mut coords = Vec::default();

    // Reference color
    let ref_color: [u8; 3] = [246, 90, 106];

    // Collect pixel clouds
    let ignore_area_bottom = 90;
    let (snd, recv) = sync_channel::<(u32, u32)>(4096);
    image
        .enumerate_rows()
        .par_bridge()
        .for_each(move |(y, row)| {
            if y > image.height() - ignore_area_bottom {
                return;
            }
            for (x, _, px) in row {
                if px.0[3] != 255 {
                    return;
                }
                if pixel_matches(&px.0, &ref_color, 2) {
                    snd.send((x, y)).unwrap();
                }
            }
        });
    while let Ok(coord) = recv.recv() {
        coords.push(coord);
    }

    // Identify target marker entities
    let target_markers = merge_cloud_into_mobs(&coords, MobType::TargetMarker);

    // Find biggest target marker
    target_markers
        .into_iter()
        .max_by_key(|x| x.name_bounds.size())
}

/// Distance: `[0..=500]`
fn find_closest_mob<'a>(
    image: &Image,
    mobs: &'a [Mob],
    avoid_bounds: Option<&Bounds>,
    max_distance: i32,
) -> Option<&'a Mob> {
    let _timer = Timer::start_new("find_closest_mob");

    // Calculate middle point of player
    let mid_x = (image.width() / 2) as i32;
    let mid_y = (image.height() / 2) as i32;

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

    // Remove mobs that are too far away
    distances = distances
        .into_iter()
        .filter(|&(_, distance)| distance <= max_distance)
        .collect();

    if let Some(avoid_bounds) = avoid_bounds {
        // Try finding closest mob that's not the mob to be avoided
        if let Some((mob, distance)) = distances
            .iter()
            .filter(|(mob, _)| {
                !avoid_bounds
                    .grow_by(25)
                    .intersects_point(&mob.get_attack_coords())
            })
            .next()
        {
            println!("Found mob avoiding last target.");
            println!("Distance: {}", distance);
            Some(mob)
        } else {
            if let Some((mob, distance)) = distances.first() {
                println!("Distance: {}", distance);
                Some(*mob)
            } else {
                None
            }
        }
    } else {
        // Return closest mob
        if let Some((mob, distance)) = distances.first() {
            println!("Distance: {}", distance);
            Some(*mob)
        } else {
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn slot_index_to_vk(index: usize) -> Vk {
    match index {
        0 => Vk::_0,
        1 => Vk::_1,
        2 => Vk::_2,
        3 => Vk::_3,
        4 => Vk::_4,
        5 => Vk::_5,
        6 => Vk::_6,
        7 => Vk::_7,
        8 => Vk::_8,
        9 => Vk::_9,
        _ => unreachable!("Invalid food index"),
    }
}

#[cfg(target_os = "windows")]
fn send_keystroke(k: Key, mode: Keymode) {
    let k = k.into();
    match mode {
        Press => winput::send(k),
        Hold => winput::press(k),
        Release => winput::release(k),
    }
}

#[cfg(target_os = "linux")]
fn send_keystroke(k: Key, mode: Keymode) {
    unimplemented!();
}

#[tauri::command]
async fn start_bot(app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();
    #[cfg(target_os = "windows")]
    let hwnd = window.hwnd().unwrap();

    let mut last_pot_time = Instant::now();
    let mut last_initial_attack_time = Instant::now();
    let mut last_kill_time = Instant::now();
    let mut last_killed_mob_bounds = Bounds::default();
    let mut last_attack_skill_usage_time = Instant::now();
    let mut is_attacking = false;
    let mut kill_count: usize = 0;
    let mut rotation_movement_tries = 0;

    std::thread::spawn(move || {
        let mut last_config_change_id = 0;
        let mut state = BotState::SearchingForEnemy;
        let mut rng = rand::thread_rng();
        let config: Arc<RwLock<BotConfig>> =
            Arc::new(RwLock::new(BotConfig::deserialize_or_default()));
        let mouse = mouse_rs::Mouse::new();

        // Listen for config changes from the UI
        let local_config = config.clone();
        app_handle.listen_global("bot_config_c2s", move |e| {
            println!("Received config change");
            if let Some(payload) = e.payload() {
                match serde_json::from_str::<BotConfig>(payload) {
                    Ok(new_config) => {
                        *local_config.write() = new_config.changed();
                    }
                    Err(e) => {
                        println!("Failed to parse bot config: {}", e);
                        println!("Payload was:\n{}", payload);
                    }
                }
            }
        });

        // Listen for bot activation state
        let local_config = config.clone();
        app_handle.listen_global("toggle_bot", move |_| {
            local_config.write().toggle_active();
        });

        // Wait a second for frontend to become ready
        std::thread::sleep(Duration::from_secs(1));

        // Send initial config to frontend
        app_handle
            .emit_all("bot_config_s2c", &*config.read())
            .unwrap();

        loop {
            let timer = Timer::start_new("loop_iter");
            let config = &*config.read();

            // Send changed config to frontend if needed
            if config.change_id() > last_config_change_id {
                config.serialize();
                app_handle.emit_all("bot_config_s2c", &config).unwrap();
                last_config_change_id = config.change_id();
            }

            // Initialize frontend info
            let mut frontend_info = FrontendInfo::new();
            frontend_info.set_is_attacking(is_attacking);
            frontend_info.set_kill_count(kill_count);

            // Update frontend info with running status
            frontend_info.set_is_running(config.is_running());

            // Check whether the bot is paused
            if !config.is_running() {
                app_handle
                    .emit_all("frontend_info", &frontend_info)
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(250));
                timer.silence();
                continue;
            }

            // Make sure the window is focused
            /*let focused_hwnd = unsafe { winapi::um::winuser::GetForegroundWindow() };
            if focused_hwnd as isize != hwnd.0 {
                app_handle
                    .emit_all("frontend_info", &frontend_info)
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
                timer.silence();
                continue;
            }*/

            // Capture the window
            
            let image = {
                let _timer = Timer::start_new("capture_window");
                #[cfg(target_os = "windows")]
                if let Ok(image) = capture_screenshot(hwnd.0) {
                    image
                } else {
                    continue;
                }
                #[cfg(target_os = "linux")]
                if let Ok(image) = capture_screenshot(&window) {
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
                if let Some(food_index) = config.get_slot_index(SlotType::Food) {
                    send_keystroke(food_index.into(), Keymode::Press);
                    last_pot_time = current_time;
                }
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
                            send_keystroke(Key::Space, Keymode::Press);
                        }
                        std::thread::sleep(idle_chunk_duration);
                    }
                    state = BotState::SearchingForEnemy;
                }
                BotState::NoEnemyFound => {
                    // Try rotating first in order to locate nearby enemies
                    if rotation_movement_tries < 20 {
                        // Rotate in random direction for a random duration
                        let key = [Key::A, Key::D].choose(&mut rng).unwrap();
                        let rotation_duration =
                            std::time::Duration::from_millis(rng.gen_range(100..250));
                        send_keystroke(*key, Keymode::Hold);
                        std::thread::sleep(rotation_duration);
                        send_keystroke(*key, Keymode::Release);
                        rotation_movement_tries += 1;
                        // Wait a bit to wait for monsters to enter view
                        std::thread::sleep(std::time::Duration::from_millis(
                            rng.gen_range(100..250),
                        ));
                        state = BotState::SearchingForEnemy;
                        continue;
                    }
                    // Check whether bot should stay in area
                    if config.should_stay_in_area() {
                        // Reset rotation movement tries to keep rotating
                        rotation_movement_tries = 0;
                        continue;
                    }
                    // If rotating multiple times failed, try other movement patterns
                    match rng.gen_range(0..3) {
                        0 => {
                            // Move into a random direction while jumping
                            let key = [Key::A, Key::D].choose(&mut rng).unwrap();
                            let rotation_duration = Duration::from_millis(rng.gen_range(100..350));
                            let movement_slices = rng.gen_range(1..4);
                            let movement_slice_duration =
                                Duration::from_millis(rng.gen_range(250..500));
                            let movement_overlap_duration =
                                movement_slice_duration.saturating_sub(rotation_duration);
                            send_keystroke(Key::W, Keymode::Hold);
                            send_keystroke(Key::Space, Keymode::Hold);
                            for _ in 0..movement_slices {
                                send_keystroke(*key, Keymode::Hold);
                                
                                std::thread::sleep(rotation_duration);
                                send_keystroke(*key, Keymode::Release);
                                std::thread::sleep(movement_overlap_duration);
                            }
                            send_keystroke(*key, Keymode::Hold);
                            std::thread::sleep(rotation_duration);
                            send_keystroke(*key, Keymode::Release);
                            send_keystroke(Key::Space, Keymode::Release);
                            send_keystroke(Key::W, Keymode::Release);
                        }
                        1 => {
                            // Move forwards while jumping
                            
                            send_keystroke(Key::W, Keymode::Hold);
                            send_keystroke(Key::Space, Keymode::Hold);
                            std::thread::sleep(std::time::Duration::from_millis(
                                rng.gen_range(1000..4000),
                            ));
                            send_keystroke(Key::Space, Keymode::Release);
                            send_keystroke(Key::W, Keymode::Release);
                        }
                        2 => {
                            // Move forwards in a slalom pattern
                            let slalom_switch_duration =
                                std::time::Duration::from_millis(rng.gen_range(350..650));
                            let total_slaloms = rng.gen_range(4..8);
                            let mut left = rng.gen_bool(0.5);
                            send_keystroke(Key::W, Keymode::Hold);
                            for _ in 0..total_slaloms {
                                let cond = if left {Key::A} else { Key::D};
                                send_keystroke(cond, Keymode::Hold);
                                std::thread::sleep(slalom_switch_duration);
                                send_keystroke(cond, Keymode::Release);
                                left = !left;
                            }
                            send_keystroke(Key::W, Keymode::Release);
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
                    state = if mobs.is_empty() {
                        // No mobs found, run movement algo
                        BotState::NoEnemyFound
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
                        let max_distance = if config.should_stay_in_area() {
                            325
                        } else {
                            1000
                        };
                        if !aggro_mobs.is_empty() {
                            println!("Found {} aggro mobs. Those die first.", aggro_mobs.len());
                            if let Some(mob) =
                                find_closest_mob(&image, aggro_mobs.as_slice(), None, max_distance)
                            {
                                BotState::EnemyFound(mob.clone())
                            } else {
                                BotState::NoEnemyFound
                            }
                        } else if !passive_mobs.is_empty() {
                            println!("Found {} passive mobs.", passive_mobs.len());
                            if let Some(mob) = {
                                // Try avoiding detection of last killed mob
                                if Instant::now().duration_since(last_kill_time)
                                    < Duration::from_secs(5)
                                {
                                    println!("Avoiding mob at {:?}", last_killed_mob_bounds);
                                    find_closest_mob(
                                        &image,
                                        passive_mobs.as_slice(),
                                        Some(&last_killed_mob_bounds),
                                        max_distance,
                                    )
                                } else {
                                    find_closest_mob(
                                        &image,
                                        passive_mobs.as_slice(),
                                        None,
                                        max_distance,
                                    )
                                }
                            } {
                                BotState::EnemyFound(mob.clone())
                            } else {
                                BotState::NoEnemyFound
                            }
                        } else {
                            println!("Mobs were found, but they're neither aggro nor neutral???");
                            BotState::NoEnemyFound
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

                        // Try to use attack skill
                        if let Some(index) =
                            config.get_random_slot_index(SlotType::AttackSkill, &mut rng)
                        {
                            // Only use attack skill if enabled and once a second at most
                            if config.should_use_attack_skills()
                                && last_attack_skill_usage_time.elapsed() > Duration::from_secs(1)
                            {
                                last_attack_skill_usage_time = Instant::now();
                                send_keystroke(index.into(), Keymode::Press);
                            }
                        }
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

                    // Check for on-demand pet config
                    if config.is_on_demand_pet() {
                        if let Some(index) = config.get_slot_index(SlotType::PickupPet) {
                            // Summon pet
                            send_keystroke(index.into(), Keymode::Press);
                            // Wait half a second to make sure everything is picked up
                            std::thread::sleep(std::time::Duration::from_millis(2000));
                            // Unsummon pet
                            send_keystroke(index.into(), Keymode::Press);
                        }
                    }
                }
                BotState::Interrupted => {
                    unimplemented!("");
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
fn capture_screenshot(window: &Window) -> Result<Image, ()> {
    //window.set().focus();
    panic!("");
    /*screenshot_rs::screenshot_window("screenshot".to_string());
    */
}
