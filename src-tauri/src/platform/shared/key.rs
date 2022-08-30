#[derive(Debug)]
pub enum KeyMode {
    Press,
    Hold,
    Release,
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Key {
    // 0-9
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    // WASD
    W,
    A,
    S,
    D,
    // INTERACTIONS
    Space,
    Escape,
    Enter,
    T,
    Left,
    Right,
}

impl From<usize> for Key {
    fn from(index: usize) -> Self {
        use Key::*;
        match index {
            0 => _0,
            1 => _1,
            2 => _2,
            3 => _3,
            4 => _4,
            5 => _5,
            6 => _6,
            7 => _7,
            8 => _8,
            9 => _9,
            _ => unreachable!("Invalid Index (expected 0-9)"),
        }
    }
}
