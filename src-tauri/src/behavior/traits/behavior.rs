use slog::Logger;
use tauri::Window;

use crate::{
    data::Target, image_analyzer::ImageAnalyzer, ipc::{BotConfig, FrontendInfo}, movement::MovementAccessor
};

pub trait Behavior<'a> {
    /// Runs on initialization
    fn new(logger: &'a Logger, movement_accessor: &'a MovementAccessor, window: &'a Window)
        -> Self;

    /// Runs on activation
    fn start(&mut self, config: &BotConfig);

    /// Runs on config change
    fn update(&mut self, config: &BotConfig);

    /// Runs when another behavior is activated
    fn stop(&mut self, config: &BotConfig);

    /// Runs when the bot is disengaged
    fn interupt(&mut self, config: &BotConfig);

    /// Runs when the bot is disengaged passing on screen targets
    fn update_targets(&mut self, _targets: Vec<Target>) {}

    fn should_update_targets(&self) -> bool {
        false
    }

    fn should_update_target_marker(&self) -> bool {
        false
    }

    fn should_update_stats(&self) -> bool {
        true
    }

    /// Runs every frame
    fn run_iteration(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        analyzer: &mut ImageAnalyzer,
    );
}
