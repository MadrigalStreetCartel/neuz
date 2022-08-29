use std::time::Instant;

use guard::guard;
use slog::Logger;

use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, ShoutConfig},
    movement::MovementAccessor,
    platform::{Key, PlatformAccessor},
    play,
};

use super::Behavior;

#[allow(dead_code)]
pub struct ShoutBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    platform: &'a PlatformAccessor<'a>,
    movement: &'a MovementAccessor/*<'a>*/,
    last_shout_time: Instant,
    shown_messages: Vec<String>,
    shout_interval: u64,
    message_iter: Option<Box<dyn Iterator<Item = String>>>,
}

impl<'a> Behavior<'a> for ShoutBehavior<'a> {
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement: &'a MovementAccessor/*<'a>*/,
    ) -> Self {
        Self {
            logger,
            platform,
            movement,
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

    fn run_iteration(&mut self, config: &BotConfig, _analyzer: &mut ImageAnalyzer) {
        let config = config.shout_config();
        self.shout(config);
    }
}

impl<'a> ShoutBehavior<'_> {
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
            PressKey(Key::Enter),
            Wait(dur::Random(100..250)),

            // Type message
            Type(message.to_string()),

            // Send message
            PressKey(Key::Enter),
            Wait(dur::Random(100..250)),

            // Close chatbox
            PressKey(Key::Escape),
            Wait(dur::Fixed(100)),
        ]);

        // Update last shout time
        self.last_shout_time = Instant::now();
    }
}
