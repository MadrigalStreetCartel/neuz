use std::{fmt, time::Instant};

use crate::{
    image_analyzer::{self, ImageAnalyzer},
    platform::{send_keystroke, Key, KeyMode},
};

#[derive(Debug, Default, Clone, Copy)]
pub enum StatusBarKind {
    #[default]
    Hp,
    Mp,
    Fp,
    Xp,
    EnemyHp,
    SpellCasting,
}
impl fmt::Display for StatusBarKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusBarKind::Hp => write!(f, "HP"),
            StatusBarKind::Mp => write!(f, "MP"),
            StatusBarKind::Fp => write!(f, "FP"),
            StatusBarKind::Xp => write!(f, "XP"),
            StatusBarKind::EnemyHp => write!(f, "enemy HP"),
            StatusBarKind::SpellCasting => write!(f, "spell cast"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StatsDetection {
    pub hp: StatInfo,
    pub mp: StatInfo,
    pub fp: StatInfo,
    pub xp: StatInfo,
    pub enemy_hp: StatInfo,
    pub spell_cast: StatInfo,

    pub stat_try_not_detected_count: i32,
}
impl StatsDetection {
    pub fn init() -> Self {
        Self {
            hp: StatInfo::new(0, 0, StatusBarKind::Hp, None),
            mp: StatInfo::new(0, 0, StatusBarKind::Mp, None),
            fp: StatInfo::new(0, 0, StatusBarKind::Fp, None),
            xp: StatInfo::new(0, 0, StatusBarKind::Xp, None),
            enemy_hp: StatInfo::new(0, 0, StatusBarKind::EnemyHp, None),
            spell_cast: StatInfo::new(0, 0, StatusBarKind::SpellCasting, None),

            stat_try_not_detected_count: 0,
        }
    }

    // update all bars values at once
    pub fn update(&mut self, image: &ImageAnalyzer) {
        self.hp.update_value(image);
        self.mp.update_value(image);
        self.fp.update_value(image);
        self.xp.update_value(image);
        self.enemy_hp.update_value(image);
        self.spell_cast.update_value(image);
    }

    // Detect whether we can read or not stat_tray and open it if needed
    pub fn detect_stat_tray(&mut self) {
        if self.hp.value == 0 && self.mp.value == 0 && self.fp.value == 0 {
            self.stat_try_not_detected_count += 1;
            if self.stat_try_not_detected_count == 3 {
                self.stat_try_not_detected_count = 0;
                send_keystroke(Key::T, KeyMode::Press);
            }
        }
    }

    // bot died
    pub fn is_alive(&mut self) -> bool {
        self.detect_stat_tray();
        // If bars are found, check if bot is alive by using hp value
        if self.hp.value == 0 {
            false
        } else {
            true
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
    pub last_item_used_time: Option<Instant>,
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
            last_value: 0,
            last_item_used_time: None,
        };
        if image.is_some() {
            res.update_value(image.unwrap());
        }
        res
    }

    pub fn update_value(&mut self, image: &ImageAnalyzer) {
        let (updated_max_w, updated_value) = image.detect_status_bar(*self).unwrap_or_default();
        let (old_max_w, old_value) = (self.max_w, self.value);

        if updated_max_w != old_max_w {
            self.max_w = updated_max_w;
        }

        if updated_value != old_value {
            // Check whever item was used soon if we don't script doesnt updated values and think we're low Hp
            if updated_value > old_value
                || self.last_item_used_time.is_some()
                    && self.last_item_used_time.unwrap().elapsed().as_millis() > 5000 // Need to implement that for healing spells
            {
                self.last_item_used_time = None;
            }

            // Update values
            self.value = updated_value;
            self.last_update_time = Some(Instant::now());
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusBarConfig {
    pub max_search_x: u32,
    pub max_search_y: u32,
    pub min_search_x: u32,
    pub min_search_y: u32,
    pub refs: [[u8; 3]; 4],
}

impl StatusBarConfig {
    pub fn new(colors: [[u8; 3]; 4]) -> Self {
        Self {
            refs: colors,
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

            Xp => StatusBarConfig::new([
                [48, 185, 244],
                [128, 212, 245],
                [52, 196, 252],
                [92, 236, 252],
            ]),
            EnemyHp => {
                let mut enemy_hp_bar = StatusBarConfig::new([
                    [174, 18, 55],
                    [188, 24, 62],
                    [204, 30, 70],
                    [220, 36, 78],
                ]);
                enemy_hp_bar.min_search_x = 310;
                enemy_hp_bar.min_search_y = 30;

                enemy_hp_bar.max_search_x = 1000;
                enemy_hp_bar.max_search_y = 60;

                enemy_hp_bar
            }
            SpellCasting => {
                let mut spell_casting_bar = StatusBarConfig::new([
                    [16, 186, 15],
                    [20, 157, 20],
                    [15, 210, 14],
                    [92, 164, 92],
                ]);
                spell_casting_bar.min_search_x = 310;
                spell_casting_bar.min_search_y = 500;
                // 800 -> 1038 fullscreen
                //
                spell_casting_bar.max_search_x = 1000;
                spell_casting_bar.max_search_y = 1080;

                spell_casting_bar
            }
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            max_search_x: 310,
            max_search_y: 120,
            min_search_x: 0,
            min_search_y: 0,
            refs: [[0; 3]; 4],
        }
    }
}

impl PartialEq for StatusBarConfig {
    fn eq(&self, other: &Self) -> bool {
        self.refs == other.refs && self.max_search_x == other.max_search_x
    }
}