use super::{ Bounds, Color, ColorDetection, MobType, PixelCloudKind };
const TARGET_BOUNDS: Bounds = Bounds {
    x: 300,
    y: 30,
    w: 550,
    h: 60,
};

const SELF_BOUNDS: Bounds = Bounds {
    x: 105,
    y: 30,
    w: 225,
    h: 110,
};

const FULL_BOUNDS: Bounds = Bounds {
    x: 0,
    y: 0,
    w: 800,
    h: 600,
};

#[derive(Debug, Copy, PartialEq, Eq, Default, Hash)]
pub enum PixelCloudKindCategorie {
    #[default]
    None,
    Mover(PixelCloudKind),
    Stat(PixelCloudKind),
}

impl Clone for PixelCloudKindCategorie {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Mover(t) => Self::Mover(t.clone()),
            Self::Stat(t) => Self::Stat(t.clone()),
        }
    }
}

impl PixelCloudKindCategorie {
    pub fn get_bounds(&self) -> Bounds {
        match self {
            Self::None => Bounds::default(),
            Self::Mover(_) => FULL_BOUNDS,
            Self::Stat(t) => {
                match t {
                    PixelCloudKind::HP(b) => {
                        match b {
                            true => TARGET_BOUNDS,
                            false => SELF_BOUNDS,
                        }
                    }
                    PixelCloudKind::MP(b) => {
                        match b {
                            true => TARGET_BOUNDS,
                            false => SELF_BOUNDS,
                        }
                    }
                    PixelCloudKind::FP => SELF_BOUNDS,
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
                    PixelCloudKind::HP(_) =>
                        Some(ColorDetection {
                            colors: vec![
                                Color::new([174, 18, 55], None),
                                Color::new([188, 24, 62], None),
                                Color::new([204, 30, 70], None),
                                Color::new([220, 36, 78], None)
                            ],
                            tolerance: 5,
                        }),
                    PixelCloudKind::MP(_) =>
                        Some(ColorDetection {
                            colors: vec![
                                Color::new([20, 84, 196], None),
                                Color::new([36, 132, 220], None),
                                Color::new([44, 164, 228], None),
                                Color::new([56, 188, 232], None)
                            ],
                            tolerance: 5,
                        }),
                    PixelCloudKind::FP =>
                        Some(ColorDetection {
                            colors: vec![
                                Color::new([45, 230, 29], None),
                                Color::new([28, 172, 28], None),
                                Color::new([44, 124, 52], None),
                                Color::new([20, 146, 20], None)
                            ],
                            tolerance: 5,
                        }),
                    _ => None,
                }
            }
        }
    }

    pub fn get_mob_colors(&self) -> Option<ColorDetection> {
        if self == &Self::Mover(PixelCloudKind::Mob(MobType::Aggressive)) {
            return Some(ColorDetection {
                colors: vec![Color::new([179, 23, 23], None)],
                tolerance: 5,
            });
        } else if self == &Self::Mover(PixelCloudKind::Mob(MobType::Passive)) {
            return Some(ColorDetection {
                colors: vec![Color::new([234, 234, 149], None)],
                tolerance: 5,
            });
        } else if self == &Self::Mover(PixelCloudKind::Mob(MobType::Violet)) {
            return Some(ColorDetection {
                colors: vec![Color::new([182, 144, 146], None)],
                tolerance: 5,
            });
        } else {
            None
        }
    }

    pub fn get_target_colors(&self) -> Option<ColorDetection> {
        if self == &Self::Mover(PixelCloudKind::Target(true)) {
            return Some(ColorDetection {
                colors: vec![Color::new([131, 148, 205], None)], // blueish
                tolerance: 5,
            });
        } else if self == &Self::Mover(PixelCloudKind::Target(false)) {
            return Some(ColorDetection {
                colors: vec![Color::new([246, 90, 106], None)], // redish
                tolerance: 5,
            });
        } else {
            None
        }
    }
}
