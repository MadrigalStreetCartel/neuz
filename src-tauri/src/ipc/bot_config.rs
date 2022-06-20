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

#[derive(Default, Serialize, Deserialize)]
pub struct VersionAgnosticBotConfig {
    change_id: u64,
    is_running: Option<bool>,
    on_demand_pet: Option<bool>,
    use_attack_skills: Option<bool>,
    slots: Option<[Slot; 10]>,
}

impl From<BotConfig> for VersionAgnosticBotConfig {
    fn from(bot_config: BotConfig) -> Self {
        Self {
            change_id: bot_config.change_id,
            is_running: Some(bot_config.is_running),
            on_demand_pet: Some(bot_config.on_demand_pet),
            use_attack_skills: Some(bot_config.use_attack_skills),
            slots: Some(bot_config.slots),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Change id to sync changes between frontend and backend
    change_id: u64,
    /// Whether the bot is running
    is_running: bool,
    /// Summon pet after kill and unsummon before next attack
    on_demand_pet: bool,
    /// Whether to use attack skills for combat
    use_attack_skills: bool,
    /// Slot configuration
    slots: [Slot; 10],
}

impl BotConfig {
    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.slot_type == slot_type)
    }

    /// Get a random matching slot index
    pub fn get_random_slot_index<R>(&self, slot_type: SlotType, rng: &mut R) -> Option<usize>
    where
        R: Rng,
    {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.slot_type == slot_type)
            .choose(rng)
            .map(|(index, _)| index)
    }

    pub fn toggle_active(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn is_on_demand_pet(&self) -> bool {
        self.on_demand_pet
    }

    pub fn should_use_attack_skills(&self) -> bool {
        self.use_attack_skills
    }

    pub fn change_id(&self) -> u64 {
        self.change_id
    }

    pub fn changed(mut self) -> Self {
        self.change_id += 1;
        self
    }

    /// Serialize config to disk
    pub fn serialize(&self) {
        let config = {
            let mut config = self.clone();
            config.is_running = false;
            config
        };
        if let Ok(mut file) = File::create(".botconfig") {
            let serializable_config = VersionAgnosticBotConfig::from(config);
            let _ = serde_json::to_writer(&mut file, &serializable_config);
        }
    }

    /// Deserialize config from disk
    pub fn deserialize_or_default() -> Self {
        if let Ok(mut file) = File::open(".botconfig") {
            let config: VersionAgnosticBotConfig = serde_json::from_reader(&mut file).unwrap_or_default();
            config.into()
        } else {
            Self::default()
        }
    }
}

impl From<VersionAgnosticBotConfig> for BotConfig {
    fn from(serialized_bot_config: VersionAgnosticBotConfig) -> Self {
        Self {
            change_id: serialized_bot_config.change_id,
            is_running: serialized_bot_config.is_running.unwrap_or(false),
            on_demand_pet: serialized_bot_config.on_demand_pet.unwrap_or(true),
            use_attack_skills: serialized_bot_config.use_attack_skills.unwrap_or(false),
            slots: serialized_bot_config.slots.unwrap_or_default(),
        }
    }
}

impl Default for BotConfig {
    fn default() -> Self {
        VersionAgnosticBotConfig::default().into()
    }
}
