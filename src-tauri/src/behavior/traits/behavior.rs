use slog::Logger;

use crate::{
    data::{StatInfo, StatsDetection, PixelDetectionInfo},
    image_analyzer::ImageAnalyzer,
    ipc::BotConfig,
    movement::MovementAccessor,
    platform::PlatformAccessor,
};

pub trait Behavior<'a> {
    /// Runs on initialization
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement_accessor: &'a MovementAccessor<'a>,
    ) -> Self;

    /// Runs on activation
    fn start(&mut self, config: &BotConfig);

    /// Runs on config change
    fn update(&mut self, config: &BotConfig);

    /// Runs on deactivation
    fn stop(&mut self, config: &BotConfig);

    /// Runs every frame
    fn run_iteration(
        &mut self,
        config: &BotConfig,
        analyzer: &ImageAnalyzer,
        stats_detection: &mut StatsDetection,
        cursor_attack: &mut PixelDetectionInfo,
    );
}
