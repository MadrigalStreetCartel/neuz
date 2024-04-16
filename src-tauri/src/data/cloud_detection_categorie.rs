use super::{ Bounds, Color, ColorDetection, MobType, CloudDetectionKind };
const TARGET_BOUNDS: Bounds = Bounds { // Monster health panel
    x: 300,
    y: 30,
    w: 550,
    h: 60,
};

const SELF_BOUNDS: Bounds = Bounds { // Player health panel aka stats tray
    x: 105,
    y: 30,
    w: 225,
    h: 110,
};

const FULL_BOUNDS: Bounds = Bounds { // Full screen not really full cause of status bar
    x: 200,
    y: 100,
    w: 800,
    h: 600,
};

#[derive(Debug, Copy, PartialEq, Eq, Default, Hash, Clone)]
pub enum CloudDetectionCategorie {
    #[default]
    None,
    Mover(CloudDetectionKind),
    Stat(CloudDetectionKind),
}

impl CloudDetectionCategorie {
    pub fn get_bounds(&self) -> Bounds {
        match self {
            Self::None => Bounds::default(),
            Self::Mover(_) => FULL_BOUNDS,
            Self::Stat(t) => {
                match t {
                    CloudDetectionKind::HP(b) => {
                        match b {
                            true => TARGET_BOUNDS,
                            false => SELF_BOUNDS,
                        }
                    }
                    CloudDetectionKind::MP(b) => {
                        match b {
                            true => TARGET_BOUNDS,
                            false => SELF_BOUNDS,
                        }
                    }
                    CloudDetectionKind::FP => SELF_BOUNDS,
                    _ => Bounds::default(),
                }
            }
        }
    }

    pub fn get_colors(&self) -> Option<ColorDetection> {
        match self {
            Self::None => None,
            Self::Mover(_) => { self.get_mob_colors().or_else(|| self.get_target_colors()) }
            Self::Stat(_) => self.get_stats_colors(),
        }
    }

    pub fn get_stats_colors(&self) -> Option<ColorDetection> {
        match self {
            Self::None => None,
            Self::Mover(_) => None,
            Self::Stat(t) => {
                match t {
                    CloudDetectionKind::HP(_) =>
                        Some(
                            ColorDetection::from(
                                vec![[174, 18, 55], [188, 24, 62], [204, 30, 70], [220, 36, 78]]
                            )
                        ),
                    CloudDetectionKind::MP(_) =>
                        Some(
                            ColorDetection::from(
                                vec![[20, 84, 196], [36, 132, 220], [44, 164, 228], [56, 188, 232]]
                            )
                        ),

                    CloudDetectionKind::FP =>
                        Some(
                            ColorDetection::from(
                                vec![[45, 230, 29], [28, 172, 28], [44, 124, 52], [20, 146, 20]]
                            )
                        ),
                    _ => None,
                }
            }
        }
    }

    pub fn get_mob_colors(&self) -> Option<ColorDetection> {
        if self == &Self::Mover(CloudDetectionKind::Mob(MobType::Aggressive)) {
            return Some(ColorDetection {
                colors: vec![Color::new([179, 23, 23], None)],
                tolerance: 5,
            });
        } else if self == &Self::Mover(CloudDetectionKind::Mob(MobType::Passive)) {
            return Some(ColorDetection {
                colors: vec![Color::new([234, 234, 149], None)],
                tolerance: 5,
            });
        } else if self == &Self::Mover(CloudDetectionKind::Mob(MobType::Violet)) {
            return Some(ColorDetection {
                colors: vec![Color::new([182, 144, 146], None)],
                tolerance: 5,
            });
        } else {
            None
        }
    }

    pub fn get_target_colors(&self) -> Option<ColorDetection> {
        if self == &Self::Mover(CloudDetectionKind::Target(true)) {
            return Some(ColorDetection {
                colors: vec![Color::new([131, 148, 205], None)], // blueish
                tolerance: 5,
            });
        } else if self == &Self::Mover(CloudDetectionKind::Target(false)) {
            return Some(ColorDetection {
                colors: vec![Color::new([246, 90, 106], None)], // redish
                tolerance: 5,
            });
        } else {
            None
        }
    }
}
