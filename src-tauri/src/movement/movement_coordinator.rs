use std::{ops::Range, thread, time::Duration};

use rand::Rng;

use crate::platform::{send_keystroke, send_message, Key, KeyMode /* , PlatformAccessor*/};

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
pub enum Movement {
    Jump,
    Move(MovementDirection, ActionDuration),
    Rotate(RotationDirection, ActionDuration),
    PressKey(Key),
    HoldKeyFor(Key, ActionDuration),
    HoldKey(Key),
    HoldKeys(Vec<Key>),
    ReleaseKey(Key),
    ReleaseKeys(Vec<Key>),
    Repeat(u64, Vec<Movement>),
    Type(String),
    Wait(ActionDuration),
}

pub struct MovementCoordinator {
    rng: rand::rngs::ThreadRng,
}

impl<'a> MovementCoordinator {
    pub fn new() -> Self {
        let rng = rand::thread_rng();

        Self {
            rng, /*, platform */
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
        M: AsRef<[Movement]>,
    {
        for movement in movements.as_ref() {
            self.play_single(movement.clone());
        }
    }

    fn play_single(&mut self, movement: Movement) {
        match movement {
            Movement::Jump => {
                send_keystroke(Key::Space, KeyMode::Press);
            }
            Movement::Move(direction, duration) => {
                let key = match direction {
                    MovementDirection::Forward => Key::W,
                    MovementDirection::Backward => Key::S,
                    MovementDirection::Random => {
                        if self.rng.gen() {
                            Key::W
                        } else {
                            Key::S
                        }
                    }
                };
                send_keystroke(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                send_keystroke(key, KeyMode::Release);
            }
            Movement::Rotate(direction, duration) => {
                let key = match direction {
                    RotationDirection::Left => Key::Left,
                    RotationDirection::Right => Key::Right,
                    RotationDirection::Random => {
                        if self.rng.gen() {
                            Key::Left
                        } else {
                            Key::Right
                        }
                    }
                };
                send_keystroke(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                send_keystroke(key, KeyMode::Release);
            }
            Movement::Wait(duration) => thread::sleep(duration.to_duration(&mut self.rng)),
            Movement::Type(text) => {
                send_message(&text);
            }
            Movement::PressKey(key) => {
                send_keystroke(key, KeyMode::Press);
            }
            Movement::HoldKeyFor(key, duration) => {
                send_keystroke(key, KeyMode::Hold);
                thread::sleep(duration.to_duration(&mut self.rng));
                send_keystroke(key, KeyMode::Release);
            }
            Movement::HoldKey(key) => {
                send_keystroke(key, KeyMode::Hold);
            }
            Movement::HoldKeys(keys) => {
                for key in keys {
                    send_keystroke(key, KeyMode::Hold);
                }
            }
            Movement::ReleaseKey(key) => {
                send_keystroke(key, KeyMode::Release);
            }
            Movement::ReleaseKeys(keys) => {
                for key in keys {
                    send_keystroke(key, KeyMode::Release);
                }
            }
            Movement::Repeat(times, movements) => {
                for _ in 0..times {
                    self.play(&movements);
                }
            }
        }
    }

    pub fn jump(&self) {
        send_keystroke(Key::Space, KeyMode::Press);
    }
}
