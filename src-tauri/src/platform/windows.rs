use winput::Vk;

use super::{Key, KeyMode};

pub const IGNORE_AREA_TOP: u32 = 0;
pub const IGNORE_AREA_BOTTOM: u32 = 80;

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

pub fn send_message(s: &str) {
    winput::send_str(s);
}
