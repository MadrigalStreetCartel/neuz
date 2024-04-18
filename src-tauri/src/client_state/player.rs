use tauri::Window;

use crate::{
    data::ProgressBar,
    platform::{eval_send_key, KeyMode},
};
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum PlayerAliveState {
    //Unknown,
    #[default]
    StatsTrayClosed,
    Alive,
    Dead,
}
#[derive(Debug, Clone)]
pub struct Player {
    pub hp: ProgressBar,
    pub mp: ProgressBar,
    pub fp: ProgressBar,
    pub is_alive: PlayerAliveState,
    pub stat_try_not_detected_count: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            hp: ProgressBar::new(0, 0),
            mp: ProgressBar::new(0, 0),
            fp: ProgressBar::new(0, 0),
            is_alive: PlayerAliveState::default(),
            stat_try_not_detected_count: 0,
        }
    }

    pub fn update(&mut self, has_tray_open: bool) {
        self.is_alive = {
            if !has_tray_open {
                PlayerAliveState::StatsTrayClosed
            } else if self.hp.value > 0 {
                PlayerAliveState::Alive
            } else {
                PlayerAliveState::Dead
            }
        };
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self, window: &Window) -> bool {
        // Since HP/MP/FP are 0 we know bar should be hidden
        if self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0 {
            self.stat_try_not_detected_count += 1;
            if self.stat_try_not_detected_count == 10 {
                self.stat_try_not_detected_count = 0;

                // Try to open char stat tray
                eval_send_key(window, "T", KeyMode::Press);
            }
            false
        } else {
            self.stat_try_not_detected_count = 0;
            true
        }
    }
}
