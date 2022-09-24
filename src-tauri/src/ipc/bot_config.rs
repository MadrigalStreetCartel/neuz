use std::{fmt, fs::File, time::Instant};

use rand::{prelude::IteratorRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Unused,
    Food,
    Pill,
    MpRestorer,
    FpRestorer,
    PickupPet,
    PickupMotion,
    AttackSkill,
    BuffSkill,
    Flying,
}
impl fmt::Display for SlotType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlotType::Food => write!(f, "food"),
            SlotType::Pill => write!(f, "pill"),
            SlotType::MpRestorer => write!(f, "mp restorer"),
            SlotType::FpRestorer => write!(f, "fp restorer"),
            SlotType::PickupPet => write!(f, "food"),
            SlotType::AttackSkill => write!(f, "food"),
            SlotType::BuffSkill => write!(f, "food"),
            SlotType::Flying => write!(f, "food"),
            _ => write!(f, "??none??"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlotBar {
    slots: Option<[Slot; 10]>,
}

impl Default for SlotBar {
    fn default() -> Self {
        Self {
            slots: Some([Slot::default(); 10]),
        }
    }
}

impl SlotBar {
    pub fn slots(&self) -> Vec<Slot> {
        return self.slots.unwrap().into_iter().collect::<Vec<_>>();
    }
    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<usize> {
        self.slots()
            .iter()
            .position(|slot| slot.slot_type == slot_type)
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index<R>(
        &self,
        slot_type: SlotType,
        rng: &mut R,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
        slot_bar_index: usize,
    ) -> Option<(usize, usize)>
    where
        R: Rng,
    {
        self.slots()
            .iter()
            .enumerate()
            .filter(|(index, slot)| {
                slot.slot_type == slot_type
                    && slot.slot_enabled
                    && slot.slot_threshold.unwrap_or(100) >= threshold.unwrap_or(0)
                    && last_slots_usage[slot_bar_index][*index].is_none()
            })
            .min_by(|x, y| x.1.slot_threshold.cmp(&y.1.slot_threshold))
            //.choose(rng)
            .map(|(index, _)| (slot_bar_index, index))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Slot {
    slot_type: SlotType,
    slot_cooldown: Option<u32>,
    slot_threshold: Option<u32>,
    slot_enabled: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            slot_type: SlotType::Unused,
            slot_cooldown: None,
            slot_threshold: None,
            slot_enabled: true,
        }
    }
}

impl Slot {
    pub fn get_slot_cooldown(&self) -> Option<u32> {
        let cooldown = self.slot_cooldown;
        if cooldown.is_some() {
            return cooldown;
        }
        return Some(1000);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BotMode {
    Farming,
    Support,
    AutoShout,
}

impl ToString for BotMode {
    fn to_string(&self) -> String {
        match self {
            BotMode::Farming => "farming",
            BotMode::Support => "support",
            BotMode::AutoShout => "auto_shout",
        }
        .to_string()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FarmingConfig {
    /// Whether the bot will try to stay in the area it started in
    stay_in_area: Option<bool>,
    /// Slot configuration
    slot_bars: Option<[SlotBar; 9]>,

    /// Disable farming
    farming_enabled: Option<bool>,

    prevent_already_attacked: Option<bool>,

    is_stop_fighting: Option<bool>,

    passive_mobs_colors: Option<[u8; 3]>,
    passive_tolerence: Option<u8>,
    aggressive_mobs_colors: Option<[u8; 3]>,
    aggressive_tolerence: Option<u8>,
}

impl FarmingConfig {
    pub fn should_stay_in_area(&self) -> bool {
        self.stay_in_area.unwrap_or(false)
    }

    pub fn get_passive_mobs_colors(&self) -> [u8; 3] {
        self.passive_mobs_colors.unwrap_or([234, 234, 149])
    }

    pub fn get_passive_tolerence(&self) -> u8 {
        self.passive_tolerence.unwrap_or(5)
    }

    pub fn get_aggressive_mobs_colors(&self) -> [u8; 3] {
        self.aggressive_mobs_colors.unwrap_or([179, 23, 23])
    }

    pub fn get_aggressive_tolerence(&self) -> u8 {
        self.aggressive_tolerence.unwrap_or(10)
    }

    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        return self.slot_bars()[slot_bar_index].slots();
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        return self.slots(slot_bar_index)[slot_index].get_slot_cooldown();
    }

    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_slot_index(slot_type);
            if found_index.is_some() {
                return Some((n, found_index.unwrap()));
            }
        }
        None
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index<R>(
        &self,
        slot_type: SlotType,
        rng: &mut R,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)>
    where
        R: Rng,
    {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                rng,
                threshold,
                last_slots_usage,
                n,
            );
            if found_index.is_some() {
                return Some(found_index.unwrap());
            }
        }
        None
    }

    /// Get a random matching slot index
    //pub fn get_random_slot_index<R>(&self, slot_type: SlotType, rng: &mut R) -> Option<usize>
    //where
    //    R: Rng,
    //{
    //    self.slots
    //        .unwrap_or_default()
    //        .iter()
    //        .enumerate()
    //        .filter(|(_, slot)| slot.slot_type == slot_type)
    //        .choose(rng)
    //        .map(|(index, _)| index)
    //}

    pub fn is_stop_fighting(&self) -> bool {
        self.is_stop_fighting.unwrap_or(false)
    }

    pub fn get_prevent_already_attacked(&self) -> bool {
        self.prevent_already_attacked.unwrap_or(false)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SupportConfig {
    slot_bars: Option<[SlotBar; 9]>,
}

impl SupportConfig {
    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        return self.slot_bars()[slot_bar_index].slots();
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        return self.slots(slot_bar_index)[slot_index].get_slot_cooldown();
    }

    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_slot_index(slot_type);
            if found_index.is_some() {
                return Some((n, found_index.unwrap()));
            }
        }
        None
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index<R>(
        &self,
        slot_type: SlotType,
        rng: &mut R,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)>
    where
        R: Rng,
    {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                rng,
                threshold,
                last_slots_usage,
                n,
            );
            if found_index.is_some() {
                return Some(found_index.unwrap());
            }
        }
        None
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShoutConfig {
    shout_interval: Option<u64>,
    shout_messages: Option<Vec<String>>,
}

impl ShoutConfig {
    pub fn shout_interval(&self) -> u64 {
        self.shout_interval.unwrap_or(60)
    }

    pub fn shout_messages(&self) -> Vec<String> {
        self.shout_messages.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Change id to sync changes between frontend and backend
    change_id: u64,

    /// Whether the bot is running
    is_running: bool,

    /// The bot mode
    mode: Option<BotMode>,

    farming_config: FarmingConfig,
    support_config: SupportConfig,
    shout_config: ShoutConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            change_id: 0,
            mode: None,
            is_running: false,
            farming_config: FarmingConfig::default(),
            support_config: SupportConfig::default(),
            shout_config: ShoutConfig::default(),
        }
    }
}

impl BotConfig {
    pub fn toggle_active(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn change_id(&self) -> u64 {
        self.change_id
    }

    pub fn changed(mut self) -> Self {
        self.change_id += 1;
        self
    }

    pub fn farming_config(&self) -> &FarmingConfig {
        &self.farming_config
    }

    pub fn support_config(&self) -> &SupportConfig {
        &self.support_config
    }

    pub fn shout_config(&self) -> &ShoutConfig {
        &self.shout_config
    }

    pub fn mode(&self) -> Option<BotMode> {
        self.mode.clone()
    }

    /// Serialize config to disk
    pub fn serialize(&self) {
        let config = {
            let mut config = self.clone();
            config.is_running = false;
            config
        };
        if let Ok(mut file) = File::create(".botconfig") {
            let _ = serde_json::to_writer(&mut file, &config);
        }
    }

    /// Deserialize config from disk
    pub fn deserialize_or_default() -> Self {
        if let Ok(mut file) = File::open(".botconfig") {
            serde_json::from_reader::<_, BotConfig>(&mut file).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
