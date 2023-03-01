use std::{fmt, time::Instant};

use palette::Hsv;
use slog::Logger;

use crate::{
    image_analyzer::{ ImageAnalyzer},
};

use super::{PointCloud, Target};

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
#[derive(Default, Debug, Clone, PartialEq)]
pub enum TargetMarkerType {
    #[default]
    None,
    Aggressive,
    Passive
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientStats {
    pub hp: StatInfo,
    pub mp: StatInfo,
    pub fp: StatInfo,
    pub target_hp: StatInfo,
    pub target_mp: StatInfo,
    pub target_marker_type: TargetMarkerType,
    pub target_marker: Option<Target>,
}
impl ClientStats {
    pub fn new() -> Self {
        Self {
            hp: StatInfo::new(0, 100, StatusBarKind::Hp, None),
            mp: StatInfo::new(0, 100, StatusBarKind::Mp, None),
            fp: StatInfo::new(0, 100, StatusBarKind::Fp, None),
            target_hp: StatInfo::new(0, 100, StatusBarKind::TargetHP, None),
            target_mp: StatInfo::new(0, 100, StatusBarKind::TargetMP, None),
            target_marker_type: TargetMarkerType::None,
            target_marker: None,
        }
    }

    // update all bars values at once
    pub fn update(&mut self, image: &mut ImageAnalyzer, logger: &Logger) {
        let should_debug = [
            self.hp.update_value(image),
            self.mp.update_value(image),
            self.fp.update_value(image),
            self.target_hp.update_value(image),
            self.target_mp.update_value(image),
            self.update_target_marker(image),

        ];

        #[cfg(debug_assertions)]
        if should_debug.contains(&true) && false  {
            self.debug_print(logger);
        }
    }

    pub fn update_target_marker(&mut self, image: &mut ImageAnalyzer) -> bool {
        let tm = image.identify_target_marker(false);

        let mut changed = false;
        if tm.0 != self.target_marker_type {
            self.target_marker_type = tm.0;
            changed = true;
        }

        if tm.1 != self.target_marker {
            self.target_marker = tm.1;
        }
        changed
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self) -> bool {
        // Since HP/MP/FP are 0 we know bar should be hidden
        return !(self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0);
    }

    // bot died
    /// Detects bot current state : 1:stats_tray_state 2:hp_state
    pub fn is_alive(&mut self) -> (bool, bool) {
        return (self.detect_stat_tray(), self.hp.value > 0);
    }
    #[cfg(debug_assertions)]
    pub fn debug_print(&mut self, logger: &Logger) {
        let target_marker_type = match self.target_marker_type {
            TargetMarkerType::Aggressive => "red",
            TargetMarkerType::Passive => "white",
            TargetMarkerType::None => "None",
        };
        slog::debug!(logger, "Stats detection"; "HP" => self.hp.value, "MP" => self.mp.value, "FP" => self.fp.value, "Target marker type" => target_marker_type, "Target HP" => self.target_hp.value, "Target MP" => self.target_mp.value);
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
        if image.is_some() {
            res.update_value(image.unwrap());
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
            None,
        );

        // Receive points from channel
        let cloud = {
            let mut cloud = PointCloud::default();
            while let Ok(point) = recv.recv() {
                cloud.push(point.0);
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
            return true;
        } else {
            return false;
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    pub max_x: u32,
    pub max_y: u32,
    pub min_x: u32,
    pub min_y: u32,
    pub refs: Vec<Hsv>,
}

impl StatusBarConfig {
    pub fn new(colors: [Hsv; 4]) -> Self {
        Self {
            refs: colors.to_vec(),
            ..Default::default()
        }
    }
}

impl From<StatusBarKind> for StatusBarConfig {
    fn from(kind: StatusBarKind) -> Self {
        use StatusBarKind::*;

        match kind {
            Hp => StatusBarConfig::new([
                Hsv::new(346.0, 0.87, 0.74),
                Hsv::new(346.0, 0.49, 0.76),
                Hsv::new(347.0, 0.81, 0.79),
                Hsv::new(345.0, 0.90, 0.58)
            ]),

            Mp => StatusBarConfig::new([
                Hsv::new(219.0, 0.89, 0.76),
                Hsv::new(214.0, 0.85, 0.80),
                Hsv::new(208.0, 0.82, 0.85),
                Hsv::new(219.0, 0.88, 0.65)
            ]),
            Fp => {
                StatusBarConfig::new([
                    Hsv::new(121.0, 0.86, 0.51),
                    Hsv::new(119.0, 0.86, 0.63),
                    Hsv::new(118.0, 0.86, 0.66),
                    Hsv::new(121.0, 0.86, 0.55)
                ])
            }

            TargetHP => {
                let mut target_hp_bar = StatusBarConfig::new([
                    Hsv::new(346.0, 0.87, 0.74),
                    Hsv::new(346.0, 0.49, 0.76),
                    Hsv::new(347.0, 0.81, 0.79),
                    Hsv::new(345.0, 0.90, 0.58)
                ]);
                target_hp_bar.min_x = 300;
                target_hp_bar.min_y = 30;

                target_hp_bar.max_x = 550;
                target_hp_bar.max_y = 60;

                target_hp_bar
            }

            TargetMP => {
                let mut target_mp_bar = StatusBarConfig::new([
                    Hsv::new(219.0, 0.89, 0.76),
                    Hsv::new(214.0, 0.85, 0.80),
                    Hsv::new(208.0, 0.82, 0.85),
                    Hsv::new(219.0, 0.88, 0.65)
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
