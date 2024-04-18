use crate::data::{ProgressBar, Target};

#[derive(Debug, Clone)]
pub struct GameTarget {
    pub hp: ProgressBar,
    pub mp: ProgressBar,
    pub is_mover: bool,
    pub is_alive: bool,
    pub is_npc: bool,
    pub is_on_screen: bool,
    pub is_red: bool,
    pub marker: Option<Target>,
    pub distance: Option<i32>,
}

impl GameTarget {
    pub fn new() -> Self {
        Self {
            hp: ProgressBar::new(0, 0),
            mp: ProgressBar::new(0, 0),
            is_mover: false,
            is_alive: false,
            is_npc: false,
            is_on_screen: false,
            is_red: false,
            marker: None,
            distance: None,
        }
    }
    pub fn update(&mut self) {
        self.is_npc = self.hp.value == 100 && self.mp.value == 0;
        self.is_mover = self.mp.value > 0;
        self.is_alive = self.hp.value > 0;
    }
}
