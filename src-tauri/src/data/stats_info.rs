use std::time::Instant;

use slog::Logger;
use tauri::Window;

use super::{
    CloudDetection, CloudDetectionCategorie, CloudDetectionKind, ProgressBar, Target
};
use crate::platform::{ eval_send_key, KeyMode };

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AliveState {
    //Unknown,
    #[default]
    StatsTrayClosed,
    Alive,
    Dead,
}

#[derive(Debug, Clone)]
pub struct ClientStats {
    pub has_tray_open: bool,
    pub hp: ProgressBar,
    pub mp: ProgressBar,
    pub fp: ProgressBar,
    pub target_hp: ProgressBar,
    pub target_mp: ProgressBar,
    pub target_is_mover: bool,
    pub target_is_npc: bool,
    pub target_is_alive: bool,
    pub target_on_screen: bool,
    pub target_marker: Option<Target>,
    pub target_distance: Option<i32>,
    pub is_alive: AliveState,
    pub stat_try_not_detected_count: i32,
    window: Window,
    _logger: Logger,
}
impl ClientStats {
    pub fn new(window: Window, logger: &Logger) -> Self {
        Self {
            has_tray_open: false,
            hp: ProgressBar::new(0, 0),
            mp: ProgressBar::new(0, 0),
            fp: ProgressBar::new(0, 0),
            is_alive: AliveState::StatsTrayClosed,
            target_hp: ProgressBar::new(0, 0),
            target_mp: ProgressBar::new(0, 0),
            target_is_mover: false,
            target_is_npc: false,
            target_is_alive: false,
            target_on_screen: false,
            target_marker: None,
            target_distance: None,
            _logger: logger.clone(),

            stat_try_not_detected_count: 0,
            window,
        }
    }

    pub fn update(&mut self, pixel_clouds: &Vec<CloudDetection>) {
        for pixel_cloud in pixel_clouds {
            match pixel_cloud.kind {
                CloudDetectionCategorie::Stat(t) => {
                    match t {
                        CloudDetectionKind::HP(is_target) => {
                            if is_target {
                                let value = pixel_cloud.process_stats(self.target_hp.max_w);
                                self.target_hp.update_value(value);
                            } else {
                                let value = pixel_cloud.process_stats(self.hp.max_w);
                                self.hp.update_value(value);
                            }
                        }
                        CloudDetectionKind::MP(is_target) => {
                            if is_target {
                                let value = pixel_cloud.process_stats(self.target_mp.max_w);
                                self.target_mp.update_value(value);
                            } else {
                                let value = pixel_cloud.process_stats(self.mp.max_w);
                                self.mp.update_value(value);
                            }
                        }
                        CloudDetectionKind::FP => {
                            let value = pixel_cloud.process_stats(self.fp.max_w);
                            self.fp.update_value(value);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.has_tray_open = self.detect_stat_tray();
        self.is_alive = {
            if !self.has_tray_open {
                AliveState::StatsTrayClosed
            } else if self.hp.value > 0 {
                AliveState::Alive
            } else {
                AliveState::Dead
            }
        };
        self.target_is_npc =
            self.target_hp.value == 100 && self.target_mp.value == 0;
        self.target_is_mover = self.target_mp.value > 0;
        self.target_is_alive = self.target_hp.value > 0;
        //self._debug_print();
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self) -> bool {
        // Since HP/MP/FP are 0 we know bar should be hidden
        if self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0 {
            self.stat_try_not_detected_count += 1;
            if self.stat_try_not_detected_count == 10 {
                self.stat_try_not_detected_count = 0;

                // Try to open char stat tray
                eval_send_key(&self.window, "T", KeyMode::Press);
            }
            false
        } else {
            self.stat_try_not_detected_count = 0;
            true
        }
    }

    pub fn _debug_print(&mut self) {
        // Stringify is_alive
        let alive_str = {
            match self.is_alive {
                AliveState::Alive => "alive",
                AliveState::Dead => "dead",
                AliveState::StatsTrayClosed => "stat tray closed",
            }
        };
        slog::debug!(self._logger, "Stats detection"; "HP" => self.hp.value, "MP" => self.mp.value, "FP" => self.fp.value, "Enemy HP" => self.target_hp.value, "Character is" => alive_str);
    }
}





