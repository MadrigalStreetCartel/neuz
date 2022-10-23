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
use ipc::FrontendInfo;
use parking_lot::RwLock;
use slog::{Drain, Level, Logger};
use tauri::Manager;

use crate::{
    behavior::{Behavior, FarmingBehavior, ShoutBehavior},
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, BotMode},
    movement::MovementAccessor,
    platform::PlatformAccessor,
    utils::Timer,
};

struct AppState {
    logger: Logger,
    image_analyzer: ImageAnalyzer,
    neuz_version: Option<[u8; 3]>,
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

    let neuz_version = {
        if neuz_version == "unknown" {
            None
        } else {
            let splitted = neuz_version.split(".").collect::<Vec<&str>>();
            let mut result: [u8; 3] = [0, 0, 0];
            result[0] = splitted[0].parse::<u8>().unwrap();
            result[1] = splitted[1].parse::<u8>().unwrap();
            result[2] = splitted[2].parse::<u8>().unwrap();
            if result[0] == 0 && result[1] == 0 && result[2] == 0 {
                None
            } else {
                Some(result)
            }
        }
    };
    // Build app
    tauri::Builder::default()
        // .menu(tauri::Menu::os_default(&context.package_info().name))
        .manage(AppState {
            logger,
            image_analyzer: ImageAnalyzer::new(),
            neuz_version: neuz_version,
        })
        .invoke_handler(tauri::generate_handler![start_bot,])
        .run(context)
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_bot(state: tauri::State<AppState>, app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("client").unwrap();
    let logger = state.logger.clone();

    let mut image_analyzer: ImageAnalyzer = state.image_analyzer.clone();

    image_analyzer.window_id = platform::get_window_id(&window).unwrap_or(0);
    let neuz_version = state.neuz_version;
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

        let send_info = |config: &FrontendInfo| {
            drop(app_handle.emit_all("bot_info_s2c", &*config) as Result<(), _>)
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
        let movement = MovementAccessor::new(/*&accessor*/);

        // Instantiate behaviors
        let mut farming_behavior = FarmingBehavior::new(&accessor, &logger, &movement);
        let mut shout_behavior = ShoutBehavior::new(&accessor, &logger, &movement);
        let mut last_mode: Option<BotMode> = None;

        let cursor_detection_js = "const overlayElem=document.createElement('div');overlayElem.style.position='absolute',overlayElem.style.left=0,overlayElem.style.top=0,overlayElem.style.height='2px',overlayElem.style.width='2px',overlayElem.style.zIndex=100,overlayElem.id='fuck',overlayElem.style.backgroundColor='red',document.body.appendChild(overlayElem),setInterval(()=>{document.body.style.cursor.indexOf('curattack')>0?overlayElem.style.backgroundColor='green':overlayElem.style.backgroundColor='red'},0.005)";
        let mut frontend_info: Arc<RwLock<FrontendInfo>> =
            Arc::new(RwLock::new(FrontendInfo::deserialize_or_default()));
        frontend_info.write().set_version(neuz_version);
        send_info(&*frontend_info.read());
        // Enter main loop
        loop {
            let timer = Timer::start_new("main_loop");
            let config = &*config.read();
            let mut frontend_info_mut = *frontend_info.read();

            // Check if cursor div is well shown
            drop(
                window.eval(
                    format!(
                        "if(!document.getElementById('fuck')){{ {} }}",
                        cursor_detection_js
                    )
                    .as_str(),
                ),
            );

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
                frontend_info_mut.set_is_running(false);
                frontend_info = Arc::new(RwLock::new(frontend_info_mut));
                // Send infos to frontend
                send_info(&*frontend_info.read());
                std::thread::sleep(std::time::Duration::from_millis(100));
                timer.silence();
                continue;
            }
            frontend_info_mut.set_is_running(true);

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

            // Capture client window
            image_analyzer.capture_window(&logger, config.farming_config());

            // Try capturing the window contents
            if image_analyzer.image_is_some() {
                // Update stats
                image_analyzer
                    .client_stats
                    .update(&image_analyzer.clone(), &logger);

                // Run the current behavior
                guard!(let Some(mode) = config.mode() else { continue; });

                // Stop bot in case of death
                let is_alive = image_analyzer.client_stats.is_alive();
                if !is_alive {
                    frontend_info_mut.set_is_alive(false);
                    frontend_info = Arc::new(RwLock::new(frontend_info_mut));
                    // Send infos to frontend
                    send_info(&*frontend_info.read());
                    continue;
                } else if is_alive && !frontend_info_mut.is_alive() {
                    frontend_info_mut.set_is_alive(true);
                }
                match mode {
                    BotMode::Farming => {
                        farming_behavior.run_iteration(
                            &mut frontend_info_mut,
                            config,
                            &mut image_analyzer,
                        );
                    }
                    BotMode::AutoShout => {
                        shout_behavior.run_iteration(
                            &mut frontend_info_mut,
                            config,
                            &mut image_analyzer,
                        );
                    }
                    _ => (),
                }
                frontend_info = Arc::new(RwLock::new(frontend_info_mut));
                // Send infos to frontend
                send_info(&*frontend_info.read());
            }

            // Update last mode
            last_mode = config.mode();
        }
    });
}
