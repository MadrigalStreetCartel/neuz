use std::fmt::Display;

use super::MobType;

#[derive(Debug, Copy, PartialEq, Eq, Hash)]
pub enum CloudDetectionKind {
    HP(bool), // bool is for whether we search for player or target stats
    MP(bool),
    FP,
    Mob(MobType),
    Target(bool), // bool is for whether we search for red or blue target
}

impl CloudDetectionKind {
    pub fn to_string(&self) -> String {
        match self {
            Self::HP(b) => {
                if *b { "HP(Target)".to_string() } else { "HP(Self)".to_string() }
            }
            Self::MP(b) => {
                if *b { "MP(Target)".to_string() } else { "MP(Self)".to_string() }
            }
            Self::FP => "FP(Self)".to_string(),
            Self::Mob(t) => {
                match t {
                    MobType::Aggressive => "Mob(Agressive)".to_string(),
                    MobType::Passive => "Mob(Passive)".to_string(),
                    MobType::Violet => "Mob(Violet)".to_string(),
                }
            }
            Self::Target(b) => {
                if *b { "Target(Red)".to_string() } else { "Target(Blue)".to_string() }
            }
        }
    }
}

impl Clone for CloudDetectionKind {
    fn clone(&self) -> Self {
        match self {
            Self::HP(t) => Self::HP(*t),
            Self::MP(t) => Self::MP(*t),
            Self::FP => Self::FP,
            Self::Mob(t) => Self::Mob(*t),
            Self::Target(t) => Self::Target(*t),
        }
    }
}

impl Display for CloudDetectionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       self.to_string().fmt(f)
    }
}
