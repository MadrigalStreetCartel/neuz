use std::{collections::HashMap, f32::consts::E, fmt, sync::mpsc::sync_channel, time::Instant};

use palette::{FromColor, Hsv, Srgb};
use slog::Logger;
use tauri::Window;

use super::{Bounds, Point, PointCloud, Target};
use crate::{
    image_analyzer::{Color, ImageAnalyzer},
    platform::{eval_send_key, KeyMode},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusBarKind {
    #[default]
    Hp,
    Mp,
    Fp,
    TargetHP,
    TargetMP,
}
impl StatusBarKind {
    pub fn update_from_bounds(&self, bounds: Option<&Bounds>, stats: &mut ClientStats) -> bool {
        match self {
            StatusBarKind::Hp => {
                return stats.hp.update_from_bound(bounds);
            }
            StatusBarKind::Mp => {
                return stats.mp.update_from_bound(bounds);
            }
            StatusBarKind::Fp => {
                return stats.fp.update_from_bound(bounds);
            }
            StatusBarKind::TargetHP => {
                return stats.target_hp.update_from_bound(bounds);
            }
            StatusBarKind::TargetMP => {
                return stats.target_mp.update_from_bound(bounds);
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AliveState {
    //Unknown,
    #[default]
    StatsTrayClosed,
    Alive,
    Dead,
}
impl fmt::Display for StatusBarKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusBarKind::Hp => write!(f, "HP"),
            StatusBarKind::Mp => write!(f, "MP"),
            StatusBarKind::Fp => write!(f, "FP"),
            StatusBarKind::TargetHP => write!(f, "enemy HP"),
            StatusBarKind::TargetMP => write!(f, "enemy MP"), // Used to be sure mob's died
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientStats {
    pub has_tray_open: bool,
    pub hp: StatInfo,
    pub mp: StatInfo,
    pub fp: StatInfo,
    pub target_hp: StatInfo,
    pub target_mp: StatInfo,
    pub target_is_mover: bool,
    pub target_is_npc: bool,
    pub target_is_alive: bool,
    pub target_on_screen: bool,
    pub target_marker: Option<Target>,
    pub target_distance: Option<i32>,
    pub is_alive: AliveState,
    pub stat_try_not_detected_count: i32,
    window: Window,
}
impl ClientStats {
    pub fn new(window: Window) -> Self {
        let res = Self {
            has_tray_open: false,
            hp: StatInfo::new(0, 100, StatusBarKind::Hp, None),
            mp: StatInfo::new(0, 100, StatusBarKind::Mp, None),
            fp: StatInfo::new(0, 100, StatusBarKind::Fp, None),
            is_alive: AliveState::StatsTrayClosed,
            target_hp: StatInfo::new(0, 0, StatusBarKind::TargetHP, None),
            target_mp: StatInfo::new(0, 0, StatusBarKind::TargetMP, None),
            target_is_mover: false,
            target_is_npc: false,
            target_is_alive: false,
            target_on_screen: false,
            target_marker: None,
            target_distance: None,

            stat_try_not_detected_count: 0,
            window,
        };
        res
    }

    // update all bars values at once
    pub fn update(&mut self, image: &ImageAnalyzer, _logger: &Logger) {
        let (hp_snd, hp_recv) = sync_channel::<Point>(4096);
        let (mp_snd, mp_recv) = sync_channel::<Point>(4096);
        let (fp_snd, fp_recv) = sync_channel::<Point>(4096);

        let (target_hp_snd, target_hp_recv) = sync_channel::<Point>(4096);
        let (target_mp_snd, target_mp_recv) = sync_channel::<Point>(4096);

        let receivers = HashMap::from([
            (StatusBarKind::Hp, hp_recv),
            (StatusBarKind::Mp, mp_recv),
            (StatusBarKind::Fp, fp_recv),
            (StatusBarKind::TargetHP, target_hp_recv),
            (StatusBarKind::TargetMP, target_mp_recv),
        ]);
        let hp_config = StatusBarConfig::get(self.hp.stat_kind);
        let mp_config = StatusBarConfig::get(self.mp.stat_kind);
        let fp_config = StatusBarConfig::get(self.fp.stat_kind);

        let target_hp_config = StatusBarConfig::get(self.target_hp.stat_kind);
        let target_mp_config = StatusBarConfig::get(self.target_mp.stat_kind);

        image.pixel_detection(&[
            (
                hp_config.bounds,
                hp_config.color,
                hp_config.tolerance,
                Box::new(move |x, y| hp_snd.send(Point::new(x, y)).unwrap()),
            ),
            (
                mp_config.bounds,
                mp_config.color,
                mp_config.tolerance,
                Box::new(move |x, y| mp_snd.send(Point::new(x, y)).unwrap()),
            ),
            (
                fp_config.bounds,
                fp_config.color,
                fp_config.tolerance,
                Box::new(move |x, y| fp_snd.send(Point::new(x, y)).unwrap()),
            ),
            (
                target_hp_config.bounds,
                target_hp_config.color,
                target_hp_config.tolerance,
                Box::new(move |x, y| target_hp_snd.send(Point::new(x, y)).unwrap()),
            ),
            (
                target_mp_config.bounds,
                target_mp_config.color,
                target_mp_config.tolerance,
                Box::new(move |x, y| target_mp_snd.send(Point::new(x, y)).unwrap()),
            ),
        ]);

        let mut recv_bounds: HashMap<StatusBarKind, PointCloud> = HashMap::default();
        for (key, recv) in receivers {
            recv_bounds.insert(key, PointCloud::default());
            while let Ok(point) = recv.recv() {
                let cloud = recv_bounds.get_mut(&key).unwrap();
                cloud.push(point);
            }
        }
        let mut should_debug = vec![];
        for (key, cloud) in recv_bounds {
            let clouds: Vec<Bounds> = cloud
                .cluster_by_distance_2d(50, 1)
                .into_iter()
                .map(|cluster| cluster.to_bounds())
                .collect();
            let cloud = clouds.first();
            let changed = key.update_from_bounds(cloud, self);
            should_debug.push(changed);
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
        self.target_is_npc = self.target_hp.value == 100 && self.target_mp.value == 0;
        self.target_is_mover = self.target_mp.value > 0;
        self.target_is_alive = self.target_hp.value > 0;
        let blue_target = image.identify_target_marker(true);
        let target = if blue_target.is_some() {
            blue_target
        } else {
            image.identify_target_marker(false)
        };
        self.target_marker = target;
        if let Some(target) = target {
            self.target_on_screen = true;
            let new_dist = image.get_target_marker_distance(target);
            if let Some(dist) = self.target_distance {
                if dist != new_dist {
                    should_debug.push(true);
                }
            }
            self.target_distance = Some(new_dist);
        } else {
            self.target_on_screen = false;
            if self.target_distance.is_some() {
                should_debug.push(true);
            }
            self.target_distance = None;
        }

        //        slog::debug!(_logger, "Stats detection"; "HP" => self.hp.value, "MP" => self.mp.value, "FP" => self.fp.value, "Enemy HP" => self.target_hp.value, "Character is" => self.is_alive(), "Enemy is NPC" => self.target_is_npc, "Enemy is Mover" => self.target_is_mover, "Enemy is alive" => self.target_is_alive, "Enemy on screen" => self.target_on_screen, "Enemy distance" => self.target_distance.unwrap_or(-1));

        // Debug is deactivated
        if should_debug.contains(&true) {
            self._debug_print(_logger);
        }
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self) -> bool {
        // Since HP/MP/FP are 0 we know bar should be hidden
        if self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0 {
            eval_send_key(&self.window, "T", KeyMode::Press);
            false
        } else {
            true
        }
    }

    pub fn _debug_print(&mut self, logger: &Logger) {
        // Stringify is_alive
        let alive_str = {
            match self.is_alive {
                AliveState::Alive => "alive",
                AliveState::Dead => "dead",
                AliveState::StatsTrayClosed => "stat tray closed",
            }
        };
        if self.is_alive == AliveState::StatsTrayClosed {
            slog::debug!(logger, "Stats tray closed");
            return;
        }
        slog::debug!(logger, "Player Stats"; "HP" => self.hp.value, "MP" => self.mp.value, "FP" => self.fp.value);
        if true || self.target_is_alive {
            slog::debug!(logger, "Target Stats"; "HP" => self.target_hp.value, "MP" => self.target_mp.value, "Is NPC" => self.target_is_npc, "Is Mover" => self.target_is_mover, "Is Alive" => self.target_is_alive, "On Screen" => self.target_on_screen, "Distance" => self.target_distance.unwrap_or(-1));
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StatInfo {
    pub max_w: u32,
    pub value: u32,
    pub stat_kind: StatusBarKind,
    pub last_value: u32,
    pub last_update_time: Option<Instant>,
}

impl PartialEq for StatInfo {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for StatInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl StatInfo {
    pub fn new(
        max_w: u32,
        value: u32,
        stat_kind: StatusBarKind,
        image: Option<&ImageAnalyzer>,
    ) -> Self {
        let mut res = Self {
            max_w,
            value,
            stat_kind,
            last_update_time: Some(Instant::now()),
            last_value: 100,
        };
        /*         if let Some(image) = image {
            res.update_value(image);
        } */

        res
    }

    pub fn reset_last_update_time(&mut self) {
        self.last_update_time = Some(Instant::now());
    }

    pub fn update_from_bound(&mut self, bounds: Option<&Bounds>) -> bool {
        if let Some(bounds) = bounds {
            let updated_max_w = bounds.w.max(self.max_w);
            let value_frac = (bounds.w as f32) / (updated_max_w as f32);
            let updated_value = ((value_frac * 100_f32) as u32).max(0).min(100);

            let (old_max_w, old_value) = (self.max_w, self.value);

            if updated_max_w != old_max_w {
                self.max_w = updated_max_w;
            }
            if updated_value != old_value {
                self.value = updated_value;
                self.last_update_time = Some(Instant::now());
                true
            } else {
                false
            }
        } else {
            if self.value != 0 {
                self.value = 0;
                self.last_update_time = Some(Instant::now());
                true
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    pub bounds: Bounds,
    pub color: Hsv,
    pub tolerance: [f32; 3],
}

impl StatusBarConfig {
    pub fn new(color: &[f32; 3], tolerance: Option<&[f32; 3]>) -> Self {
        Self {
            color: Hsv::new(color[0], color[1], color[2]),
            tolerance: tolerance.unwrap_or(&[0.0, 0.0, 0.0]).to_owned(),
            ..Default::default()
        }
    }
}

impl StatusBarConfig {
    #[inline]
    pub fn get(kind: StatusBarKind) -> Self {
        kind.into()
    }
}

impl From<StatusBarKind> for StatusBarConfig {
    fn from(kind: StatusBarKind) -> Self {
        use StatusBarKind::*;
        let target_bounds = Bounds {
            x: 330,
            y: 30,
            w: 185,
            h: 30,
        };
        match kind {
            Hp => StatusBarConfig::new(&[346.2, 0.85, 0.8], Some(&[2.0, 0.01, 0.13])),

            Mp => StatusBarConfig::new(&[208.0, 0.82, 0.85], Some(&[2.0, 0.01, 0.13])),
            Fp => StatusBarConfig::new(&[117.4, 0.85, 0.75], Some(&[1.0, 0.02, 0.9])),

            TargetHP => {
                let mut target_hp_bar =
                    StatusBarConfig::new(&[346.2, 0.85, 0.8], Some(&[2.0, 0.01, 0.13]));
                target_hp_bar.bounds = target_bounds;

                target_hp_bar
            }

            TargetMP => {
                let mut target_mp_bar =
                    StatusBarConfig::new(&[208.0, 0.82, 0.85], Some(&[2.0, 0.01, 0.13]));
                target_mp_bar.bounds = target_bounds;

                target_mp_bar
            }
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            bounds: Bounds {
                x: 105,
                y: 35,
                w: 110,
                h: 65,
            },
            color: Hsv::default(),
            tolerance: [0.0, 0.0, 0.0],
        }
    }
}
