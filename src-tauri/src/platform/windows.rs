use std::time::Duration;

use winput::Vk;

use super::{Key, KeyMode};

pub const IGNORE_AREA_TOP: u32 = 0;

#[allow(clippy::just_underscores_and_digits)]
impl From<Key> for Vk {
    fn from(k: Key) -> Self {
        use Key::*;
        match k {
            _0 => Vk::_0,
            _1 => Vk::_1,
            _2 => Vk::_2,
            _3 => Vk::_3,
            _4 => Vk::_4,
            _5 => Vk::_5,
            _6 => Vk::_6,
            _7 => Vk::_7,
            _8 => Vk::_8,
            _9 => Vk::_9,
            W => Vk::W,
            A => Vk::A,
            S => Vk::S,
            D => Vk::D,
            Space => Vk::Space,
            Escape => Vk::Escape,
            Enter => Vk::Enter,
            T => Vk::T,
            Left => Vk::LeftArrow,
            Right => Vk::RightArrow,
            _F1 => Vk::F1,
            _F2 => Vk::F2,
            _F3 => Vk::F3,
            _F4 => Vk::F4,
            _F5 => Vk::F5,
            _F6 => Vk::F6,
            _F7 => Vk::F7,
            _F8 => Vk::F8,
            _F9 => Vk::F9,
        }
    }
}

pub fn send_keystroke(k: Key, mode: KeyMode) {
    let k: Vk = k.into();
    match mode {
        KeyMode::Press => winput::send(k),
        KeyMode::Hold => winput::press(k),
        KeyMode::Release => winput::release(k),
    }
}

pub fn send_slot(slot_bar_index: usize, k: Key) {
    let k: Vk = k.into();

    let slot_bar_key: Key = format!("F{}", slot_bar_index + 1).as_str().into();
    let slot_bar_vk: Vk = slot_bar_key.into();

    winput::send(slot_bar_vk);
    winput::send(k);
    std::thread::sleep(Duration::from_millis(200));
}

pub fn send_message(s: &str) {
    winput::send_str(s);
}
