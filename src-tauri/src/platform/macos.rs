use enigo::{Enigo, KeyboardControllable};

use super::{Key, KeyMode};

pub const IGNORE_AREA_TOP: u32 = 60;

#[allow(clippy::just_underscores_and_digits)]
impl From<Key> for enigo::Key {
    fn from(k: Key) -> Self {
        use Key::*;
        match k {
            _0 => enigo::Key::Layout('0'),
            _1 => enigo::Key::Layout('1'),
            _2 => enigo::Key::Layout('2'),
            _3 => enigo::Key::Layout('3'),
            _4 => enigo::Key::Layout('4'),
            _5 => enigo::Key::Layout('5'),
            _6 => enigo::Key::Layout('6'),
            _7 => enigo::Key::Layout('7'),
            _8 => enigo::Key::Layout('8'),
            _9 => enigo::Key::Layout('9'),
            W => enigo::Key::Layout('w'),
            A => enigo::Key::Layout('a'),
            S => enigo::Key::Layout('s'),
            D => enigo::Key::Layout('d'),
            Space => enigo::Key::Space,
            Esc => enigo::Key::Escape,
            Enter => enigo::Key::Return,
            T => enigo::Key::Layout('T'),
            Left => enigo::Key::LeftArrow,
            Right => enigo::Key::RightArrow,
        }
    }
}

pub fn send_keystroke(k: Key, mode: KeyMode) {
    let k: enigo::Key = k.into();
    let mut enigo = Enigo::new();
    match mode {
        KeyMode::Press => enigo.key_click(k),
        KeyMode::Hold => enigo.key_down(k),
        KeyMode::Release => enigo.key_up(k),
    }
}

pub fn send_slot(k:Key) {
    let k: enigo::Key = k.into();
    let mut enigo = Enigo::new();
    enigo.key_click(k);
    std::thread::sleep(Duration::from_millis(500));
}

pub fn send_message(text: &str) {
    let mut enigo = Enigo::new();
    enigo.key_sequence(text);
}
