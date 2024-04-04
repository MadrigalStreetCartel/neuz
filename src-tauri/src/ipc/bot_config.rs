use std::{fmt, fs::File, time::Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Unused,
    Food,
    Pill,
    HealSkill,
    AOEHealSkill,
    AOEAttackSkill,
    MpRestorer,
    FpRestorer,
    PickupPet,
    PickupMotion,
    AttackSkill,
    BuffSkill,
    RezSkill,
    Flying,
    PartySkill,
}
impl fmt::Display for SlotType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlotType::Food => write!(f, "food"),
            SlotType::Pill => write!(f, "pill"),
            SlotType::MpRestorer => write!(f, "mp restorer"),
            SlotType::FpRestorer => write!(f, "fp restorer"),
            SlotType::PickupPet => write!(f, "pickup pet"),
            SlotType::AttackSkill => write!(f, "attack skill"),
            SlotType::AOEAttackSkill => write!(f, "aoe attack skill"),
            SlotType::BuffSkill => write!(f, "buff skill"),
            SlotType::RezSkill => write!(f, "rez skill"),
            SlotType::Flying => write!(f, "fly"),
            SlotType::PartySkill => write!(f, "PartySkill"),
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
        self.slots.unwrap().into_iter().collect::<Vec<_>>()
    }
    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<usize> {
        self.slots()
            .iter()
            .position(|slot| slot.slot_type == slot_type)
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
        slot_bar_index: usize,
    ) -> Option<(usize, usize)> {
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

    /// Get all usable slots for an index
    pub fn get_all_usable_slots_for_index(
        &self,
        slot_type: SlotType,
        slot_bar_index: usize,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Vec<(usize, usize)> {
        let mut all_valid_slots: Vec<(usize, usize)> = Vec::new();
        for (index, current_slot) in self.slots().iter().enumerate() {
            if current_slot.slot_enabled
                && current_slot.slot_type == slot_type
                && last_slots_usage[slot_bar_index][index].is_none()
            {
                all_valid_slots.push((slot_bar_index, index))
            }
        }
        all_valid_slots
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
        Some(100)
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
    /// Slot configuration
    slot_bars: Option<[SlotBar; 9]>,

    /// Search for mob circle pattern
    circle_pattern_rotation_duration: Option<u64>,

    /// Disable farming
    farming_enabled: Option<bool>,

    prevent_already_attacked: Option<bool>,
    prioritize_aggro: Option<bool>,

    is_stop_fighting: Option<bool>,

    passive_mobs_colors: Option<[Option<u8>; 3]>,
    passive_tolerence: Option<u8>,

    aggressive_mobs_colors: Option<[Option<u8>; 3]>,
    aggressive_tolerence: Option<u8>,

    violet_mobs_colors: Option<[Option<u8>; 3]>,
    violet_tolerence: Option<u8>,

    obstacle_avoidance_cooldown: Option<u64>,
    obstacle_avoidance_max_try: Option<u32>,

    min_mobs_name_width: Option<u32>,
    max_mobs_name_width: Option<u32>,

    min_hp_attack: Option<u32>,
    on_death_disconnect: Option<bool>,
    interval_between_buffs: Option<u64>,
    mobs_timeout: Option<u64>,
    aoe_farming: Option<u32>,

    on_afk_disconnect: Option<bool>,
    afk_timeout: Option<u64>,
}

impl FarmingConfig {
    pub fn on_afk_disconnect(&self) -> bool {
        self.on_afk_disconnect.unwrap_or(false)
    }
    pub fn afk_timeout(&self) -> u128 {
        self.afk_timeout.unwrap_or(3000).into()
    }

    pub fn mobs_timeout(&self) -> u128 {
        self.mobs_timeout.unwrap_or(0).into()
    }

    pub fn interval_between_buffs(&self) -> u128 {
        self.interval_between_buffs.unwrap_or(2000).into()
    }

    pub fn on_death_disconnect(&self) -> bool {
        self.on_death_disconnect.unwrap_or(true)
    }

    pub fn circle_pattern_rotation_duration(&self) -> u64 {
        self.circle_pattern_rotation_duration.unwrap_or(30)
    }

    pub fn obstacle_avoidance_cooldown(&self) -> u128 {
        self.obstacle_avoidance_cooldown.unwrap_or(5000).into()
    }

    pub fn obstacle_avoidance_max_try(&self) -> u32 {
        self.obstacle_avoidance_max_try.unwrap_or(5)
    }

    pub fn min_mobs_name_width(&self) -> u32 {
        self.min_mobs_name_width.unwrap_or(11)
    }

    pub fn max_mobs_name_width(&self) -> u32 {
        self.max_mobs_name_width.unwrap_or(180)
    }

    pub fn min_hp_attack(&self) -> u32 {
        self.min_hp_attack.unwrap_or(0)
    }
    pub fn max_aoe_farming(&self) -> u32 {
        self.aoe_farming.unwrap_or(1)
    }

    pub fn passive_mobs_colors(&self) -> [Option<u8>; 3] {
        self.passive_mobs_colors.unwrap_or([None, None, None])
    }

    pub fn passive_tolerence(&self) -> u8 {
        self.passive_tolerence.unwrap_or(5)
    }

    pub fn aggressive_mobs_colors(&self) -> [Option<u8>; 3] {
        self.aggressive_mobs_colors.unwrap_or([None, None, None])
    }
    pub fn aggressive_tolerence(&self) -> u8 {
        self.aggressive_tolerence.unwrap_or(10)
    }

    pub fn violet_mobs_colors(&self) -> [Option<u8>; 3] {
        self.violet_mobs_colors.unwrap_or([None, None, None])
    }
    pub fn violet_tolerence(&self) -> u8 {
        self.violet_tolerence.unwrap_or(10)
    }

    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        self.slot_bars()[slot_bar_index].slots()
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        self.slots(slot_bar_index)[slot_index].get_slot_cooldown()
    }

    /// Get the first matching slot index
    pub fn slot_index(&self, slot_type: SlotType) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_slot_index(slot_type);
            if let Some(found_index) = found_index {
                return Some((n, found_index));
            }
        }
        None
    }

    ///Get a list of usable matching slot index types for the farming behavior
    pub fn get_all_usable_slot_for_type(
        &self,
        slot_type: SlotType,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Vec<(usize, usize)> {
        let mut all_valid_slots: Vec<(usize, usize)> = Vec::new();
        for slot_bar_index in 0..9 {
            let result = self.slot_bars()[slot_bar_index].get_all_usable_slots_for_index(
                slot_type,
                slot_bar_index,
                last_slots_usage,
            );
            for found_skill in result {
                all_valid_slots.push((slot_bar_index, found_skill.1));
            }
        }
        all_valid_slots
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                threshold,
                last_slots_usage,
                n,
            );
            if let Some(found_index) = found_index {
                return Some(found_index);
            }
        }
        None
    }

    pub fn is_stop_fighting(&self) -> bool {
        self.is_stop_fighting.unwrap_or(false)
    }

    pub fn prevent_already_attacked(&self) -> bool {
        self.prevent_already_attacked.unwrap_or(true)
    }
    pub fn prioritize_aggro(&self) -> bool {
        self.prioritize_aggro.unwrap_or(true)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SupportConfig {
    slot_bars: Option<[SlotBar; 9]>,
    obstacle_avoidance_cooldown: Option<u64>,
    on_death_disconnect: Option<bool>,
    on_afk_disconnect: Option<bool>,
    is_in_party: Option<bool>,
    afk_timeout: Option<u64>,
    interval_between_buffs: Option<u64>,
}

impl SupportConfig {
    pub fn interval_between_buffs(&self) -> u128 {
        self.interval_between_buffs.unwrap_or(2000).into()
    }

    pub fn on_death_disconnect(&self) -> bool {
        self.on_death_disconnect.unwrap_or(true)
    }
    pub fn on_afk_disconnect(&self) -> bool {
        self.on_afk_disconnect.unwrap_or(false)
    }
    pub fn is_in_party(&self) -> bool {
        self.is_in_party.unwrap_or(false)
    }

    pub fn afk_timeout(&self) -> u128 {
        self.afk_timeout.unwrap_or(3000).into()
    }

    pub fn obstacle_avoidance_cooldown(&self) -> u128 {
        self.obstacle_avoidance_cooldown.unwrap_or(0).into()
    }

    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        self.slot_bars()[slot_bar_index].slots()
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        self.slots(slot_bar_index)[slot_index].get_slot_cooldown()
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                threshold,
                last_slots_usage,
                n,
            );
            if let Some(found_index) = found_index {
                return Some(found_index);
            }
        }
        None
    }

    ///Get a list of usable matching slot index types for the support behavior
    pub fn get_all_usable_slot_for_type(
        &self,
        slot_type: SlotType,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Vec<(usize, usize)> {
        let mut all_valid_slots: Vec<(usize, usize)> = Vec::new();

        for slot_bar_index in 0..9 {
            let result = self.slot_bars()[slot_bar_index].get_all_usable_slots_for_index(
                slot_type,
                slot_bar_index,
                last_slots_usage,
            );
            for found_skill in result {
                all_valid_slots.push((slot_bar_index, found_skill.1));
            }
        }
        all_valid_slots
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShoutConfig {
    shout_interval: Option<u64>,
    shout_messages: Option<Vec<String>>,

    on_afk_disconnect: Option<bool>,
    afk_timeout: Option<u64>,
}

impl ShoutConfig {
    pub fn on_afk_disconnect(&self) -> bool {
        self.on_afk_disconnect.unwrap_or(false)
    }
    pub fn afk_timeout(&self) -> u128 {
        self.afk_timeout.unwrap_or(3000).into()
    }

    pub fn shout_interval(&self) -> u64 {
        self.shout_interval.unwrap_or(30000)
    }

    pub fn shout_messages(&self) -> Vec<String> {
        self.shout_messages.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub fn serialize(&self, path: String) {
        let config = {
            let mut config = self.clone();
            config.is_running = false;
            config
        };
        if let Ok(mut file) = File::create(path) {
            let _ = serde_json::to_writer(&mut file, &config);
        }
    }

    /// Deserialize config from disk
    pub fn deserialize_or_default(path: String) -> Self {
        if let Ok(mut file) = File::open(path) {
            serde_json::from_reader::<_, BotConfig>(&mut file).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
