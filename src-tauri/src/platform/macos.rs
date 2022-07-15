use enigo::{Enigo, KeyboardControllable};

use super::{Key, KeyMode};

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
            _10 => enigo::Key::Layout('w'),
            A => enigo::Key::Layout('a'),
            S => enigo::Key::Layout('s'),
            D => enigo::Key::Layout('d'),
            Space => enigo::Key::Space,
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