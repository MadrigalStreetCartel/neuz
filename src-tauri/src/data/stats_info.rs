use std::{fmt, time::Instant};

use slog::Logger;
use tauri::Window;

use super::PointCloud;
use crate::{
    image_analyzer::{Color, ImageAnalyzer},
    platform::{eval_send_key, KeyMode},
};

#[derive(Debug, Default, Clone, Copy)]
pub enum StatusBarKind {
    #[default]
    Hp,
    Mp,
    Fp,
    TargetHP,
    TargetMP,
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
    pub hp: StatInfo,
    pub mp: StatInfo,
    pub fp: StatInfo,
    pub target_hp: StatInfo,
    pub target_mp: StatInfo,
    pub target_is_mover: bool,
    pub target_is_npc: bool,
    pub target_is_alive: bool,
    pub target_on_screen: bool,
    pub target_distance: Option<i32>,
    is_alive: bool,
    pub stat_try_not_detected_count: i32,
    window: Window,
}
impl ClientStats {
    pub fn new(window: Window) -> Self {
        Self {
            hp: StatInfo::new(0, 100, StatusBarKind::Hp, None),
            mp: StatInfo::new(0, 100, StatusBarKind::Mp, None),
            fp: StatInfo::new(0, 100, StatusBarKind::Fp, None),
            is_alive: true,
            target_hp: StatInfo::new(0, 0, StatusBarKind::TargetHP, None),
            target_mp: StatInfo::new(0, 0, StatusBarKind::TargetMP, None),
            target_is_mover: false,
            target_is_npc: false,
            target_is_alive: false,
            target_on_screen: false,
            target_distance: None,

            stat_try_not_detected_count: 0,
            window,
        }
    }

    // update all bars values at once
    pub fn update(&mut self, image: &ImageAnalyzer, _logger: &Logger) {
        let _should_debug = [
            self.hp.update_value(image),
            self.mp.update_value(image),
            self.fp.update_value(image),
            self.target_hp.update_value(image),
            self.target_mp.update_value(image),
        ];

        self.target_is_npc = self.target_hp.value == 100 && self.target_mp.value == 0;
        self.target_is_mover = image.client_stats.target_mp.value > 0;
        self.target_is_alive = self.target_hp.value > 0;
        let target = image.identify_target_marker(true);
        self.target_on_screen = self.target_is_mover && target.is_some();
        if self.target_on_screen {
            self.target_distance = Some(image.get_target_marker_distance(target.unwrap()));
        } else {
            self.target_distance = None;
        }

        // Debug is deactivated
        /*if should_debug.contains(&true) {
            self.debug_print(_logger);
        }*/
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self) -> bool {
        // Since HP/MP/FP are 0 we know bar should be hidden
        if self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0 {
            self.stat_try_not_detected_count += 1;
            if self.stat_try_not_detected_count == 5 {
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

    // bot died
    pub fn is_alive(&mut self) -> bool {
        // We need to be sure that char tray is open before
        if self.detect_stat_tray() {
            // Obfviously
            self.is_alive = self.hp.value != 0;
        }
        self.is_alive
    }

    pub fn _debug_print(&mut self, logger: &Logger) {
        // Stringify is_alive
        let alive_str = {
            if self.is_alive() {
                "alive"
            } else {
                "dead"
            }
        };
        slog::debug!(logger, "Stats detection"; "HP" => self.hp.value, "MP" => self.mp.value, "FP" => self.fp.value, "Enemy HP" => self.target_hp.value, "Character is" => alive_str);
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
        if let Some(image) = image {
            res.update_value(image);
        }

        res
    }

    pub fn reset_last_update_time(&mut self) {
        self.last_update_time = Some(Instant::now());
    }

    pub fn update_value(&mut self, image: &ImageAnalyzer) -> bool {
        let status_bar_config: StatusBarConfig = self.stat_kind.into();
        let recv = image.pixel_detection(
            status_bar_config.refs,
            status_bar_config.min_x,
            status_bar_config.min_y,
            status_bar_config.max_x,
            status_bar_config.max_y,
            Some(2),
        );

        // Receive points from channel
        let cloud = {
            let mut cloud = PointCloud::default();
            while let Ok(point) = recv.recv() {
                cloud.push(point);
            }
            cloud
        };

        // Calculate bounds
        let bounds = cloud.to_bounds();

        // Recalculate value tracking info
        let updated_max_w = bounds.w.max(self.max_w);
        let value_frac = bounds.w as f32 / updated_max_w as f32;
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
    }
}

#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    pub max_x: u32,
    pub max_y: u32,
    pub min_x: u32,
    pub min_y: u32,
    pub refs: Vec<Color>,
}

impl StatusBarConfig {
    pub fn new(colors: [[u8; 3]; 4]) -> Self {
        Self {
            refs: colors
                .iter()
                .map(|v| Color::new(v[0], v[1], v[2]))
                .collect(),
            ..Default::default()
        }
    }
}

impl From<StatusBarKind> for StatusBarConfig {
    fn from(kind: StatusBarKind) -> Self {
        use StatusBarKind::*;

        match kind {
            Hp => {
                StatusBarConfig::new([[174, 18, 55], [188, 24, 62], [204, 30, 70], [220, 36, 78]])
            }

            Mp => StatusBarConfig::new([
                [20, 84, 196],
                [36, 132, 220],
                [44, 164, 228],
                [56, 188, 232],
            ]),
            Fp => {
                StatusBarConfig::new([[45, 230, 29], [28, 172, 28], [44, 124, 52], [20, 146, 20]])
            }

            TargetHP => {
                let mut target_hp_bar = StatusBarConfig::new([
                    [174, 18, 55],
                    [188, 24, 62],
                    [204, 30, 70],
                    [220, 36, 78],
                ]);
                target_hp_bar.min_x = 300;
                target_hp_bar.min_y = 30;

                target_hp_bar.max_x = 550;
                target_hp_bar.max_y = 60;

                target_hp_bar
            }

            TargetMP => {
                let mut target_mp_bar = StatusBarConfig::new([
                    [20, 84, 196],
                    [36, 132, 220],
                    [44, 164, 228],
                    [56, 188, 232],
                ]);
                target_mp_bar.min_x = 300;
                target_mp_bar.min_y = 50;

                target_mp_bar.max_x = 550;
                target_mp_bar.max_y = 60;

                target_mp_bar
            }
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            max_x: 225,
            max_y: 110,
            min_x: 105,
            min_y: 30,
            refs: vec![],
        }
    }
}

impl PartialEq for StatusBarConfig {
    fn eq(&self, other: &Self) -> bool {
        /*self.refs == other.refs &&*/
        self.max_x == other.max_x
    }
}
