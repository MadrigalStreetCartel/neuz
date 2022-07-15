#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{sync::Arc, time::Duration};

use behavior::{Behavior, FarmingBehavior};
use image_analyzer::ImageAnalyzer;
use libscreenshot::WindowCaptureProvider;
use parking_lot::RwLock;
use platform::PlatformAccessor;
use tauri::{Manager, Window};

mod behavior;
mod data;
mod image_analyzer;
mod ipc;
mod platform;
mod utils;

use crate::{ipc::BotConfig, utils::Timer};

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        // .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![start_bot,])
        .run(context)
        .expect("error while running tauri application");
}

/// Capture the current window contents.
fn capture_window(window: &Window) -> Option<ImageAnalyzer> {
    let _timer = Timer::start_new("capture_window");
    if let Some(provider) = libscreenshot::get_window_capture_provider() {
        if let Some(window_id) = platform::get_window_id(window) {
            if let Ok(image) = provider.capture_window(window_id) {
                Some(ImageAnalyzer::new(image))
            } else {
                println!("Capturing window failed.");
                None
            }
        } else {
            println!("Obtaining window handle failed.");
            None
        }
    } else {
        None
    }
}

#[tauri::command]
fn start_bot(app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();

    std::thread::spawn(move || {
        let mut last_config_change_id = 0;
        let config: Arc<RwLock<BotConfig>> =
            Arc::new(RwLock::new(BotConfig::deserialize_or_default()));

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

        let send_config = |config: &BotConfig| {
            drop(app_handle.emit_all("bot_config_s2c", &*config) as Result<(), _>)
        };

        // NOTE: Currently not needed.
        // let send_frontend_info = |frontend_info: &FrontendInfo| {
        //     drop(app_handle.emit_all("bot_info_s2c", frontend_info) as Result<(), _>)
        // };

        // Wait a second for frontend to become ready
        std::thread::sleep(Duration::from_secs(1));

        // Send initial config to frontend
        send_config(&*config.read());

        // Create platform accessor
        let accessor = PlatformAccessor {
            window: &window,
            mouse: mouse_rs::Mouse::new(),
        };

        // Instantiate behaviors
        let mut farming_behavior = FarmingBehavior::new(&accessor);

        // Enter main loop
        loop {
            let timer = Timer::start_new("main_loop");
            let config = &*config.read();

            // Send changed config to frontend if needed
            if config.change_id() > last_config_change_id {
                config.serialize();
                send_config(config);
                last_config_change_id = config.change_id();
            }

            // Continue early if the bot is not engaged
            if !config.is_running() {
                // send_frontend_info(&frontend_info);
                std::thread::sleep(std::time::Duration::from_millis(250));
                timer.silence();
                continue;
            }

            // Try again a bit later if the window is not focused
            if !platform::get_window_focused(&window) {
                // send_frontend_info(&frontend_info);
                std::thread::sleep(std::time::Duration::from_millis(100));
                timer.silence();
                continue;
            }

            // Try capturing the window contents
            if let Some(image_analyzer) = capture_window(&window) {
                // Run the current behavior
                farming_behavior.run_iteration(config, image_analyzer);
            }

            // Update frontend info and send it over
            // frontend_info.set_kill_count(kill_count);
            // send_frontend_info(&frontend_info);
        }
    });
}
