use serde::{Deserialize, Serialize};

use crate::data::Bounds;

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone)]
pub struct FrontendInfo {
    /* enemy_bounds: Option<Vec<Bounds>>,
    active_enemy_bounds: Option<Bounds>, */
    enemy_kill_count: u32,
    kill_min_avg: f32,
    kill_hour_avg: f32,
    is_attacking: bool,
    is_running: bool,
}

impl FrontendInfo {
    pub fn new() -> Self {
        Self {
            /* enemy_bounds: None,
            active_enemy_bounds: None, */
            enemy_kill_count: 0,
            kill_min_avg: 0.0,
            kill_hour_avg: 0.0,
            is_attacking: false,
            is_running: false,
        }
    }

   /*  pub fn set_enemy_bounds(&mut self, enemy_bounds: Vec<Bounds>) {
        self.enemy_bounds = Some(enemy_bounds);
    }

    pub fn set_active_enemy_bounds(&mut self, active_enemy_bounds: Bounds) {
        self.active_enemy_bounds = Some(active_enemy_bounds);
    }
 */
    pub fn set_kill_count(&mut self, enemy_kill_count: u32) {
        self.enemy_kill_count = enemy_kill_count;
    }

    pub fn set_kill_avg(&mut self, active_enemy_bounds: (f32, f32)) {
        self.kill_min_avg = active_enemy_bounds.0;
        self.kill_hour_avg = active_enemy_bounds.1;

    }
    pub fn set_is_attacking(&mut self, is_attacking: bool) {
        self.is_attacking = is_attacking;
    }

    pub fn set_is_running(&mut self, is_running: bool) {
        self.is_running = is_running;
    }

    /// Serialize config to disk
    pub fn serialize(&self) {
        let config = {
            let mut config = self.clone();
            config
        };
    }

    /// Deserialize config from disk
    pub fn deserialize_or_default() -> Self {

        Self::default()

    }
}
