use std::{ops::Range, thread, time::Duration};

use crate::platform::{ /* PlatformAccessor, */ KeyMode, KeyManager};
use rand::Rng;
use tauri::Window;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum MovementDirection {
    Forward,
    Backward,
    Random,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum RotationDirection {
    Left,
    Right,
    Random,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ActionDuration {
    Fixed(u64),
    Random(Range<u64>),
}

impl ActionDuration {
    fn to_duration(&self, rng: &mut rand::rngs::ThreadRng) -> Duration {
        match self {
            Self::Fixed(ms) => Duration::from_millis(*ms),
            Self::Random(range) => Duration::from_millis(rng.gen_range(range.clone())),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Movement<'a> {
    Jump,
    Move(MovementDirection, ActionDuration),
    Rotate(RotationDirection, ActionDuration),
    PressKey(&'a str),
    HoldKeyFor(&'a str, ActionDuration),
    HoldKey(&'a str),
    HoldKeys(Vec<&'a str>),
    ReleaseKey(&'a str),
    ReleaseKeys(Vec<&'a str>),
    Repeat(u64, Vec<Movement<'a>>),
    Type(String),
    Wait(ActionDuration),
}

pub struct MovementCoordinator {
    rng: rand::rngs::ThreadRng,
    key_manager: KeyManager,
}

impl<'a> MovementCoordinator {
    pub fn new(key_manager: KeyManager) -> Self {
        let rng = rand::thread_rng();

        Self {
            rng, /*, platform */
            key_manager,
        }
    }

    // Wrapper functions

    /*   pub fn with_probability<F>(&mut self, probability: f64, func: F)
    where
        F: Fn(&Self),
    {
        if self.rng.gen_bool(probability) {
            func(self);
        }
    } */

    // Movement functions

    pub fn play<M>(&mut self, movements: M)
    where
        M: AsRef<[Movement<'a>]>,
    {
        for movement in movements.as_ref() {
            self.play_single(movement.clone());
        }
    }

    fn play_single(&mut self, movement: Movement) {
        match movement {
            Movement::Jump => {
                self.key_manager.eval_send_key("Space", KeyMode::Hold);
                std::thread::sleep(std::time::Duration::from_millis(500));
                self.key_manager.eval_send_key("Space", KeyMode::Release);
            }
            Movement::Move(direction, duration) => {
                let key = match direction {
                    MovementDirection::Forward => "W",
                    MovementDirection::Backward => "S",
                    MovementDirection::Random => {
                        if self.rng.gen() {
                            "W"
                        } else {
                            "S"
                        }
                    }
                };
                self.key_manager.eval_send_key(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                self.key_manager.eval_send_key(key, KeyMode::Release);
            }
            Movement::Rotate(direction, duration) => {
                let key = match direction {
                    RotationDirection::Left => "Left",
                    RotationDirection::Right => "Right",
                    RotationDirection::Random => {
                        if self.rng.gen() {
                            "Left"
                        } else {
                            "Right"
                        }
                    }
                };
                self.key_manager.eval_send_key(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                self.key_manager.eval_send_key(key, KeyMode::Release);
            }
            Movement::Wait(duration) => thread::sleep(duration.to_duration(&mut self.rng)),
            Movement::Type(text) => {
                self.key_manager.eval_send_message(&text);
            }
            Movement::PressKey(key) => {
                self.key_manager.eval_send_key(key, KeyMode::Press);
            }
            Movement::HoldKeyFor(key, duration) => {
                self.key_manager.eval_send_key(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                self.key_manager.eval_send_key(key, KeyMode::Release);
            }
            Movement::HoldKey(key) => {
                self.key_manager.eval_send_key(key, KeyMode::Hold);
            }
            Movement::HoldKeys(keys) => {
                for key in keys {
                    self.key_manager.eval_send_key(key, KeyMode::Hold);
                }
            }
            Movement::ReleaseKey(key) => {
                self.key_manager.eval_send_key(key, KeyMode::Release);
            }
            Movement::ReleaseKeys(keys) => {
                for key in keys {
                    self.key_manager.eval_send_key(key, KeyMode::Release);
                }
            }
            Movement::Repeat(times, movements) => {
                for _ in 0..times {
                    self.play(&movements);
                }
            }
        }
    }
}
