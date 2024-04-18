use slog::Logger;
use tauri::Window;

use crate::{
    cloud_detection::{CloudDetection, CloudDetectionKind, CloudDetectionType},
    data::Target,
    platform::{eval_send_key, KeyMode},
};

use super::{
    player::{Player, PlayerAliveState},
    target::GameTarget,
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AliveState {
    //Unknown,
    #[default]
    StatsTrayClosed,
    Alive,
    Dead,
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub has_tray_open: bool,
    pub player: Player,
    pub target: GameTarget,
    pub found_targets: Vec<Target>,
    window: Window,
    _logger: Logger,
}
impl ClientState {
    pub fn new(window: Window, logger: &Logger) -> Self {
        Self {
            has_tray_open: false,
            player: Player::new(),
            target: GameTarget::new(),
            found_targets: vec![],
            _logger: logger.clone(),

            window,
        }
    }

    pub fn update(&mut self, pixel_clouds: &Vec<CloudDetection>) {
        for pixel_cloud in pixel_clouds {
            let is_player = pixel_cloud.zone.is("Player");
            match pixel_cloud.kind {
                CloudDetectionType::Stat(t) => match t {
                    CloudDetectionKind::Hp => {
                        if !is_player {
                            let value = pixel_cloud.process_stats(self.target.hp.max_w);
                            self.target.hp.update_value(value);
                        } else {
                            let value = pixel_cloud.process_stats(self.player.hp.max_w);
                            self.player.hp.update_value(value);
                        }
                    }
                    CloudDetectionKind::Mp => {
                        if !is_player {
                            let value = pixel_cloud.process_stats(self.target.mp.max_w);
                            self.target.mp.update_value(value);
                        } else {
                            let value = pixel_cloud.process_stats(self.player.mp.max_w);
                            self.player.mp.update_value(value);
                        }
                    }
                    CloudDetectionKind::Fp => {
                        let value = pixel_cloud.process_stats(self.player.fp.max_w);
                        self.player.fp.update_value(value);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let has_tray_open = self.player.detect_stat_tray(&self.window);
        self.player.update(has_tray_open);
        self.target.update();

        //self._debug_print();
    }

    pub fn _debug_print(&mut self) {
        // Stringify is_alive
        let alive_str = {
            match self.player.is_alive {
                PlayerAliveState::Alive => "alive",
                PlayerAliveState::Dead => "dead",
                PlayerAliveState::StatsTrayClosed => "stat tray closed",
            }
        };
        slog::debug!(self._logger, "Stats detection"; "HP" => self.player.hp.value, "MP" => self.player.mp.value, "FP" => self.player.fp.value, "Enemy HP" => self.target.hp.value, "Character is" => alive_str);
    }
}
