use std::time::{Duration, Instant};

use rand::{Rng};

use crate::{
    image_analyzer::{ImageAnalyzer},
    ipc::{BotConfig, ShoutConfig},
    platform::{send_keystroke, send_message, Key, KeyMode, PlatformAccessor},
};

use super::Behavior;

#[derive(Debug, Clone, Copy)]
enum State {
    IdleShout,
    Shout,
    //Moving,
}

pub struct ShoutBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    platform: &'a PlatformAccessor<'a>,
    state: State,
    last_shout_time: Instant,
    shown_messages: Vec<String>,
    left_messages: Vec<String>

}

impl<'a> Behavior<'a> for ShoutBehavior<'a> {
    fn new(platform: &'a PlatformAccessor<'a>) -> Self {
        Self {
            platform,
            rng: rand::thread_rng(),
            state: State::Shout,
            last_shout_time: Instant::now(),
            shown_messages: Vec::new(),
            left_messages:Vec::new()
        }
    }

    fn start(&mut self) {}

    fn stop(&mut self) {}

    fn run_iteration(&mut self, config: &BotConfig, analyzer:Option<ImageAnalyzer>) {
        let config = config.shout_config();
        // Check state machine
        self.state = match self.state {
            State::IdleShout => self.on_idle(config),
            State::Shout => self.on_auto_shout(config),
        }
    }
}

impl<'a> ShoutBehavior<'_> {

    fn on_idle(&mut self, _config: &ShoutConfig) -> State {

            let total_idle_duration = Duration::from_secs(_config.shout_interval());
            let idle_chunks = self.rng.gen_range(1..4);
            let idle_chunk_duration = total_idle_duration / idle_chunks;
            
            // Do mostly nothing, but jump sometimes
            for _ in 0..idle_chunks {
                if self.rng.gen_bool(0.1) {
                    send_keystroke(Key::Space, KeyMode::Press);
                }
                std::thread::sleep(idle_chunk_duration);
            }
    
            // Transition to next state
            State::Shout
    }

    fn on_auto_shout(&mut self, config: &ShoutConfig) -> State {
        self.last_shout_time = Instant::now();
        // If it's first try or all messages have been displayed
        if self.left_messages.len() == 0 || self.left_messages.len() == self.shown_messages.len() {
            self.left_messages = config.shout_message().clone();
        }

        // Randomly selected message and push it to shown_messages
        let message:&str = &self.left_messages[self.rng.gen_range(0.. self.left_messages.len())].clone();
        self.shown_messages.push(message.to_string());

        // filter left_messages to update
        self.left_messages = self.left_messages.clone()
        .into_iter()
        .filter_map(|s|if s != message { Some(s)}else{None})
        .collect();
        
        // Open chatbox
        send_keystroke(Key::Enter, KeyMode::Press);
        std::thread::sleep( Duration::from_millis(self.rng.gen_range(1..10)));

        // Write message
        send_message(message);
        //std::thread::sleep( Duration::from_millis(self.rng.gen_range(1..10)));

        // Send it
        send_keystroke(Key::Enter, KeyMode::Press);
        //std::thread::sleep( Duration::from_millis(self.rng.gen_range(1..10)));
        
        // Closing chatbox
        send_keystroke(Key::Escape, KeyMode::Press);
        std::thread::sleep( Duration::from_millis(100));
        

        if self.rng.gen_bool(0.9) {
            State::IdleShout
        } else {
            State::Shout
        }
    }
}
