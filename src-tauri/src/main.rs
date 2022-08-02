#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod behavior;
mod data;
mod image_analyzer;
mod ipc;
mod movement;
mod platform;
mod utils;

use std::{sync::Arc, time::Duration};

use guard::guard;
use libscreenshot::WindowCaptureProvider;
use parking_lot::RwLock;
use slog::{Drain, Level, Logger};
use tauri::{Manager, Window};

use crate::{
    behavior::{Behavior, FarmingBehavior, ShoutBehavior},
    image_analyzer::{ImageAnalyzer, StatInfo},
    ipc::{BotConfig, BotMode},
    movement::MovementAccessor,
    platform::{send_keystroke, Key, KeyMode, PlatformAccessor},
    utils::Timer,
};

struct AppState {
    logger: Logger,
    hp: StatInfo,
    mp: StatInfo,
    fp: StatInfo,
    enemy_hp: StatInfo,
    spell_cast: StatInfo,
    is_alive: bool,
    bars_not_detected_warn_count: i32,
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
        .manage(AppState {
            logger,
            hp: StatInfo::default(),
            mp: StatInfo::default(),
            fp: StatInfo::default(),
            enemy_hp: StatInfo::default(),
            spell_cast: StatInfo::default(),
            is_alive: true,
            bars_not_detected_warn_count: 0,
        })
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

    // Stats
    let mut hp = state.hp.clone();
    let mut mp = state.mp.clone();
    let mut fp = state.fp.clone();
    let mut enemy_hp = state.enemy_hp.clone();
    let mut spel_cast = state.spell_cast.clone();

    let mut is_alive = state.is_alive.clone();
    let mut bars_not_detected_warn_count = state.bars_not_detected_warn_count.clone();

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

        // Create movement accessor
        let movement = MovementAccessor::new(&accessor);

        // Instantiate behaviors
        let mut farming_behavior = FarmingBehavior::new(&accessor, &logger, &movement);
        let mut shout_behavior = ShoutBehavior::new(&accessor, &logger, &movement);
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

                // Check HP/MP/FP values and store them
                hp = image_analyzer
                    .detect_status_bar(hp, image_analyzer::StatusBarKind::Hp)
                    .unwrap_or_default();
                mp = image_analyzer
                    .detect_status_bar(mp, image_analyzer::StatusBarKind::Mp)
                    .unwrap_or_default();
                fp = image_analyzer
                    .detect_status_bar(fp, image_analyzer::StatusBarKind::Fp)
                    .unwrap_or_default();
                enemy_hp = image_analyzer
                    .detect_status_bar(enemy_hp, image_analyzer::StatusBarKind::EnemyHp)
                    .unwrap_or_default();
                spel_cast = StatInfo::default()/*image_analyzer
                    .detect_status_bar(spel_cast, image_analyzer::StatusBarKind::SpellCasting)
                    .unwrap_or_default()*/;

                // Check whether bars are displayed
                if hp.value == 0 && mp.value == 0 && fp.value == 0 {
                    slog::warn!(logger, "Stat tray not detected";"bars_not_detected_warn_count" => bars_not_detected_warn_count);
                    bars_not_detected_warn_count += 1;
                    if bars_not_detected_warn_count == 3 {
                        bars_not_detected_warn_count = 0;
                        slog::warn!(logger, "Trying to open stat tray");
                        send_keystroke(Key::T, KeyMode::Press);
                    }
                } else {
                    // If bars are found, check if bot is alive by using hp value
                    if hp.value == 0 {
                        if is_alive {
                            slog::info!(logger, "Bot died");
                        }
                        is_alive = false;
                    } else {
                        if !is_alive {
                            slog::info!(logger, "Bot respawned");
                        }
                        is_alive = true;
                    }
                }

                if is_alive {
                    match mode {
                        BotMode::Farming => {
                            farming_behavior.run_iteration(
                                config,
                                &image_analyzer,
                                hp,
                                mp,
                                fp,
                                enemy_hp,
                                spel_cast,
                            );
                        }
                        BotMode::AutoShout => {
                            shout_behavior.run_iteration(
                                config,
                                &image_analyzer,
                                hp,
                                mp,
                                fp,
                                enemy_hp,
                                spel_cast,
                            );
                        }
                        _ => (),
                    }
                }
            }

            // Update last mode
            last_mode = config.mode();
        }
    });
}
