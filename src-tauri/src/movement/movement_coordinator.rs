use std::{ops::Range, thread, time::Duration};

use rand::Rng;
use tauri::Window;

use crate::platform::{
    eval_send_message, /* , PlatformAccessor*/
    KeyMode, eval_send_key,
};

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
    window: Window,
}

impl<'a> MovementCoordinator {
    pub fn new(window: Window) -> Self {
        let rng = rand::thread_rng();

        Self {
            rng, /*, platform */
            window
        }
    }

    // Wrapper functions

    pub fn with_probability<F>(&mut self, probability: f64, func: F)
    where
        F: Fn(&Self),
    {
        if self.rng.gen_bool(probability) {
            func(self);
        }
    }

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
                eval_send_key(&self.window, "Space", KeyMode::Press);
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
                eval_send_key(&self.window, key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                eval_send_key(&self.window, key, KeyMode::Release);
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
                eval_send_key(&self.window, key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                eval_send_key(&self.window, key, KeyMode::Release);
            }
            Movement::Wait(duration) => thread::sleep(duration.to_duration(&mut self.rng)),
            Movement::Type(text) => {
                eval_send_message(&self.window,&text);
            }
            Movement::PressKey(key) => {
                eval_send_key(&self.window, key, KeyMode::Press);
            }
            Movement::HoldKeyFor(key, duration) => {
                eval_send_key(&self.window, key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                eval_send_key(&self.window, key, KeyMode::Release);
            }
            Movement::HoldKey(key) => {
                eval_send_key(&self.window, key, KeyMode::Hold);
            }
            Movement::HoldKeys(keys) => {
                for key in keys {
                    eval_send_key(&self.window, key, KeyMode::Hold);
                }
            }
            Movement::ReleaseKey(key) => {
                eval_send_key(&self.window, key, KeyMode::Release);
            }
            Movement::ReleaseKeys(keys) => {
                for key in keys {
                    eval_send_key(&self.window, key, KeyMode::Release);
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
