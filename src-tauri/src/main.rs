#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{sync::Arc, time::Duration};

use behavior::{Behavior, FarmingBehavior, ShoutBehavior};
use guard::guard;
use image_analyzer::ImageAnalyzer;
use libscreenshot::WindowCaptureProvider;
use parking_lot::RwLock;
use platform::PlatformAccessor;
use slog::{Drain, Level, Logger};
use tauri::{Manager, Window};

mod behavior;
mod data;
mod image_analyzer;
mod ipc;
mod platform;
mod utils;

use crate::{
    ipc::{BotConfig, BotMode},
    utils::Timer,
};

struct AppState {
    logger: Logger,
}

fn main() {
    // Generate tauri context
    let context = tauri::generate_context!();
    let neuz_version = context
        .config()
        .package
        .version
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Setup logging
    let sentry_options = sentry::ClientOptions {
        release: Some(std::borrow::Cow::from(format!("v{}", neuz_version))),
        server_name: Some(std::borrow::Cow::from(format!("neuz v{}", neuz_version))),
        ..Default::default()
    };
    let _sentry = sentry::init((
        "https://ce726b0d4b634de8a44ab5564bc924fe@o1322474.ingest.sentry.io/6579555",
        sentry_options,
    ));
    let drain = {
        let decorator = slog_term::TermDecorator::new().stdout().build();
        let drain = slog_term::CompactFormat::new(decorator)
            .build()
            .filter_level(Level::Trace)
            .fuse();
        slog_async::Async::new(drain).build().fuse()
    };
    let drain = sentry_slog::SentryDrain::new(drain).fuse();
    let logger = Logger::root(drain.fuse(), slog::o!());

    // Build app
    tauri::Builder::default()
        // .menu(tauri::Menu::os_default(&context.package_info().name))
        .manage(AppState { logger })
        .invoke_handler(tauri::generate_handler![start_bot,])
        .run(context)
        .expect("error while running tauri application");
}

/// Capture the current window contents.
fn capture_window(logger: &Logger, window: &Window) -> Option<ImageAnalyzer> {
    let _timer = Timer::start_new("capture_window");
    if let Some(provider) = libscreenshot::get_window_capture_provider() {
        if let Some(window_id) = platform::get_window_id(window) {
            if let Ok(image) = provider.capture_window(window_id) {
                Some(ImageAnalyzer::new(image))
            } else {
                slog::warn!(logger, "Failed to capture window"; "window_id" => window_id);
                None
            }
        } else {
            slog::warn!(logger, "Failed to obtain window id");
            None
        }
    } else {
        None
    }
}

#[tauri::command]
fn start_bot(state: tauri::State<AppState>, app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();
    let logger = state.logger.clone();

    std::thread::spawn(move || {
        let logger = logger.clone();
        let mut last_config_change_id = 0;
        let config: Arc<RwLock<BotConfig>> =
            Arc::new(RwLock::new(BotConfig::deserialize_or_default()));

        // Listen for config changes from the UI
        let local_config = config.clone();
        let logger_botconfig_c2s = logger.clone();
        app_handle.listen_global("bot_config_c2s", move |e| {
            slog::trace!(logger_botconfig_c2s, "Received config change"; "event_payload" => e.payload());
            if let Some(payload) = e.payload() {
                match serde_json::from_str::<BotConfig>(payload) {
                    Ok(new_config) => {
                        *local_config.write() = new_config.changed();
                    }
                    Err(e) => {
                        slog::error!(logger_botconfig_c2s, "Failed to parse config change"; "error" => e.to_string(), "error_payload" => payload);
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
        let mut farming_behavior = FarmingBehavior::new(&accessor, &logger);
        let mut shout_behavior = ShoutBehavior::new(&accessor, &logger);
        let mut last_mode: Option<BotMode> = None;

        // Enter main loop
        loop {
            let timer = Timer::start_new("main_loop");
            let config = &*config.read();

            // Send changed config to frontend if needed
            if config.change_id() > last_config_change_id {
                config.serialize();
                send_config(config);
                last_config_change_id = config.change_id();

                // Update behaviors
                farming_behavior.update(config);
                shout_behavior.update(config);
            }

            // Continue early if the bot is not engaged
            if !config.is_running() {
                std::thread::sleep(std::time::Duration::from_millis(250));
                timer.silence();
                continue;
            }

            // Try again a bit later if the window is not focused
            if !platform::get_window_focused(&window) {
                std::thread::sleep(std::time::Duration::from_millis(100));
                timer.silence();
                continue;
            }

            // Make sure an operation mode is set
            guard!(let Some(mode) = config.mode() else {
                std::thread::sleep(std::time::Duration::from_millis(100));
                timer.silence();
                continue;
            });

            // Check if mode is different from last mode
            if let Some(last_mode) = last_mode.as_ref() {
                if &mode != last_mode {
                    slog::info!(logger, "Mode changed"; "old_mode" => last_mode.to_string(), "new_mode" => mode.to_string());

                    // Stop all behaviors
                    farming_behavior.stop(&config);
                    shout_behavior.stop(&config);

                    // Start the current behavior
                    match mode {
                        BotMode::Farming => farming_behavior.start(&config),
                        BotMode::AutoShout => shout_behavior.start(&config),
                        _ => (),
                    }
                }
            }

            // Try capturing the window contents
            if let Some(image_analyzer) = capture_window(&logger, &window) {
                // Run the current behavior
                guard!(let Some(mode) = config.mode() else { continue; });

                match mode {
                    BotMode::Farming => {
                        farming_behavior.run_iteration(config, &image_analyzer);
                    }
                    BotMode::AutoShout => {
                        shout_behavior.run_iteration(config, &image_analyzer);
                    }
                    _ => (),
                }
            }

            // Update last mode
            last_mode = config.mode();
        }
    });
}
