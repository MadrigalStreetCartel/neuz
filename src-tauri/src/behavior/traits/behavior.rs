use std::time::{Instant, Duration};

use slog::Logger;
use tauri::{Window};

use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, SlotType},
    movement::MovementAccessor, platform::{KeyManager},
};

pub trait Behavior<'a> {

    /// Runs on initialization
    fn new(logger: &'a Logger, movement_accessor: &'a MovementAccessor, key_manager: &'a KeyManager)
        -> Self;

    /// Runs on activation
    fn start(&mut self, config: &BotConfig);

    /// Runs on config change
    fn update(&mut self, config: &BotConfig);

    /// Runs on deactivation
    fn stop(&mut self, config: &BotConfig);

    /// Runs every frame
    fn run_iteration(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        analyzer: &mut ImageAnalyzer,
    );
}

pub struct SlotsUsage <'a> {
    key_manager: &'a KeyManager,
    bot_config: Option<BotConfig>,
    config_type: String,
    last_usage: [[Option<Instant>; 10]; 9],
    last_buff_usage: Instant,
}

impl<'a> SlotsUsage <'a> {
    pub fn new(key_manager: &'a KeyManager, config_type: String) -> Self {
        Self {
            key_manager,
            bot_config: None,
            config_type,
            last_usage: [[None; 10]; 9],
            last_buff_usage: Instant::now(),
        }
    }

    pub fn update_config(&mut self, bot_config: BotConfig) {
        self.bot_config = Some(bot_config);
    }

    pub fn reset_slots_usage(&mut self) {
        self.last_usage = [[None; 10]; 9];
    }
    /// Update slots cooldown timers
    pub fn update_slots_usage(&mut self) {
        if let Some(config) = &self.bot_config {

            let mut slotbar_index = 0;
            for slot_bars in self.last_usage {
                let mut slot_index = 0;
                for last_time in slot_bars {
                    let cooldown = {
                        if self.config_type == "Farming" {
                            config.farming_config().get_slot_cooldown(slotbar_index, slot_index).unwrap_or(100).try_into()
                        } else {
                            config.support_config().get_slot_cooldown(slotbar_index, slot_index).unwrap_or(100).try_into()
                        }
                    };
                    if last_time.is_some() && cooldown.is_ok() {
                        let slot_last_time = last_time.unwrap().elapsed().as_millis();
                        if slot_last_time > cooldown.unwrap() {
                            self.last_usage[slotbar_index][slot_index] = None;
                        }
                    }
                    slot_index += 1;
                }
                slotbar_index += 1;
                drop(slot_index);
            }
            drop(slotbar_index);
        }
    }

    pub fn get_slot_for(
        &mut self,
        threshold: Option<u32>,
        slot_type: SlotType,
        send: bool,
    ) -> Option<(usize, usize)> {
        if let Some(config) = &self.bot_config {
            let slot = {
                if self.config_type == "Farming" {
                    config.farming_config().get_usable_slot_index(slot_type, threshold, self.last_usage)
                } else {
                    config.support_config().get_usable_slot_index(slot_type, threshold, self.last_usage)
                }
            };
            if let Some(slot_index) = slot {
                if send {
                    self.send_slot(slot_index);
                }

                return Some(slot_index);
            }
        }
        return None;
    }

    pub fn send_slot(&mut self, slot_index: (usize, usize)) {
        // Send keystroke for first slot mapped to pill
        self.key_manager.send_slot_eval(slot_index.0, slot_index.1);
        // Update usage last time
        self.last_usage[slot_index.0][slot_index.1] = Some(Instant::now());
    }

    pub fn check_buffs(&mut self) {
        if let Some(config) = &self.bot_config {
            let interval_between_buffs = config.interval_between_buffs();
            if self.last_buff_usage.elapsed().as_millis() > interval_between_buffs {
                self.last_buff_usage = Instant::now();
                self.get_slot_for( None, SlotType::BuffSkill, true);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    pub fn check_restorations(&mut self, image: &mut ImageAnalyzer) {
        // Check HP
        let stat = Some(image.client_stats.hp.value);
        if image.client_stats.hp.value > 0 {
            if self.get_slot_for(stat, SlotType::Pill, true)
                .is_none()
            {
                self.get_slot_for( stat, SlotType::Food, true);
            }
        }

        //Check target HP
        if self.config_type == "Support" {
            let stat = Some(image.client_stats.target_hp.value);
            if image.client_stats.target_hp.value > 0 {
                self.get_slot_for( stat, SlotType::HealSkill, true);
            }
        }


        // Check MP
        let stat = Some(image.client_stats.mp.value);
        if image.client_stats.mp.value > 0 {
            self.get_slot_for( stat, SlotType::MpRestorer, true);
        }

        // Check FP
        let stat = Some(image.client_stats.fp.value);
        if image.client_stats.fp.value > 0 {
            self.get_slot_for( stat, SlotType::FpRestorer, true);
        }
    }
}


