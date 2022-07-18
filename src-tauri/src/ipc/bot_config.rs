use std::fs::File;

use rand::{prelude::IteratorRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Unused,
    Food,
    PickupPet,
    AttackSkill,
    BuffSkill,
    Flying,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Slot {
    slot_type: SlotType,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            slot_type: SlotType::Unused,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum BotMode {
    Farming,
    Support,
    AutoShout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FarmingConfig {
    /// Summon pet after kill and unsummon before next attack
    on_demand_pet: Option<bool>,
    /// Whether to use attack skills for combat
    use_attack_skills: Option<bool>,
    /// Whether the bot will try to stay in the area it started in
    stay_in_area: Option<bool>,
    /// Slot configuration
    slots: Option<[Slot; 10]>,
    /// Disable farming
    farming_enabled: Option<bool>,
}

impl FarmingConfig {
    pub fn should_use_on_demand_pet(&self) -> bool {
        self.on_demand_pet.unwrap_or(false)
    }

    pub fn should_use_attack_skills(&self) -> bool {
        self.use_attack_skills.unwrap_or(false)
    }

    pub fn should_stay_in_area(&self) -> bool {
        self.stay_in_area.unwrap_or(false)
    }

    pub fn slots(&self) -> Vec<Slot> {
        self.slots
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [Slot::default(); 10].into_iter().collect::<Vec<_>>())
    }

    pub fn farming_enabled(&self) -> bool {
        self.farming_enabled.unwrap_or(true)
    }

    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<usize> {
        self.slots
            .unwrap_or_default()
            .iter()
            .position(|slot| slot.slot_type == slot_type)
    }

    /// Get a random matching slot index
    pub fn get_random_slot_index<R>(&self, slot_type: SlotType, rng: &mut R) -> Option<usize>
    where
        R: Rng,
    {
        self.slots
            .unwrap_or_default()
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.slot_type == slot_type)
            .choose(rng)
            .map(|(index, _)| index)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SupportConfig {
    slots: Option<[Slot; 10]>,
}

impl SupportConfig {
    pub fn slots(&self) -> Vec<Slot> {
        self.slots
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [Slot::default(); 10].into_iter().collect::<Vec<_>>())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShoutConfig {
    shout_interval: Option<u64>,
    shout_message: Option<Vec<String>>,
    shout_enabled: Option<bool>,
}

impl ShoutConfig {
    pub fn shout_interval(&self) -> u64 {
        self.shout_interval.unwrap_or(60)
    }

    pub fn shout_message(&self) -> Vec<String> {
        self.shout_message.clone().unwrap_or_default()
    }

    pub fn shout_enabled(&self) -> bool {
        self.shout_enabled.unwrap_or(false)
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
