use std::time::Instant;

use guard::guard;
use slog::Logger;
use tauri::Window;

use super::Behavior;
use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, ShoutConfig},
    movement::MovementAccessor,
    play,
};

#[allow(dead_code)]
pub struct ShoutBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    movement: &'a MovementAccessor,
    window: &'a Window,
    last_shout_time: Instant,
    shown_messages: Vec<String>,
    shout_interval: u64,
    message_iter: Option<Box<dyn Iterator<Item = String>>>,
}

impl<'a> Behavior<'a> for ShoutBehavior<'a> {
    fn new(logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            logger,
            movement,
            window,
            rng: rand::thread_rng(),
            last_shout_time: Instant::now(),
            shown_messages: Vec::new(),
            shout_interval: 30000,
            message_iter: None,
        }
    }

    fn start(&mut self, config: &BotConfig) {
        self.update(config);
    }

    fn update(&mut self, config: &BotConfig) {
        let config = config.shout_config();
        self.shown_messages = config.shout_messages();
        self.message_iter = Some(Box::new(self.shown_messages.clone().into_iter().cycle()));
        self.shout_interval = config.shout_interval();
    }

    fn stop(&mut self, _config: &BotConfig) {
        self.message_iter = None;
    }

    fn run_iteration(
        &mut self,
        _frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        _analyzer: &mut ImageAnalyzer,
    ) {
        let config = config.shout_config();
        self.shout(config);
    }
}

impl ShoutBehavior<'_> {
    fn shout(&mut self, _config: &ShoutConfig) {
        use crate::movement::prelude::*;

        // Return early if time since last shout is less than shout interval
        if Instant::now()
            .duration_since(self.last_shout_time)
            .as_millis()
            < self.shout_interval as u128
        {
            return;
        }

        // Find next message to shout
        guard!(let Some(mut messages) = self.message_iter.as_mut() else { return });
        guard!(let Some(message) = messages.next() else { return });

        // Avoid sending empty messages
        if message.trim().is_empty() {
            return;
        }

        // Log message
        slog::debug!(self.logger, "Shouting"; "message" => &message);

        // Play movement
        play!(self.movement => [
            // Open chatbox
            PressKey("Enter"),
            Wait(dur::Random(100..250)),

            // Type message
            Type(message.to_string()),
            Wait(dur::Random(100..200)),

            // Send message
            PressKey("Enter"),
            Wait(dur::Random(100..250)),

            // Close chatbox
            PressKey("Escape"),
            Wait(dur::Fixed(100)),
        ]);

        // Update last shout time
        self.last_shout_time = Instant::now();
    }
}
