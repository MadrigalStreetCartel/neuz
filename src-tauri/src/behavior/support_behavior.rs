use std::time::Instant;

use slog::Logger;
use tauri::Window;

use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, SlotType, SupportConfig},
    movement::MovementAccessor,
    platform::send_slot_eval,
    play,
};

use super::Behavior;

pub struct SupportBehavior<'a> {
    movement: &'a MovementAccessor,
    window: &'a Window,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_buff_usage: Instant,
    last_jump_time: Instant,
    //is_on_flight: bool,
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(_logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            movement,
            window,
            slots_usage_last_time: [[None; 10]; 9],
            last_buff_usage: Instant::now(),
            last_jump_time: Instant::now(),
            //is_on_flight: false,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {
        self.slots_usage_last_time = [[None; 10]; 9];
    }

    fn run_iteration(
        &mut self,
        _frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer,
    ) {
        let config = config.support_config();

        self.update_slots_usage(config);

        if image.client_stats.target_hp.value == 0 && image.identify_target_marker(config).is_some() {
            self.get_slot_for(config, None, SlotType::RezSkill, true);
            self.slots_usage_last_time = [[None; 10]; 9];
            return;
        }
        self.check_restorations(config, image);
        if image.client_stats.target_hp.value > 0 {
            self.check_buffs(config);

            use crate::movement::prelude::*;

            play!(self.movement => [
                PressKey("Z"),
            ]);

            if config.jump_cooldown() > 0 {
                if self.last_jump_time.elapsed().as_millis() > config.jump_cooldown() {
                    self.last_jump_time = Instant::now();
                    play!(self.movement => [
                        Jump,
                    ]);
                }
            }
        }
    }
}

impl<'a> SupportBehavior<'_> {
    /// Update slots cooldown timers
    fn update_slots_usage(&mut self, config: &SupportConfig) {
        let mut slotbar_index = 0;
        for slot_bars in self.slots_usage_last_time {
            let mut slot_index = 0;
            for last_time in slot_bars {
                let cooldown = config
                    .get_slot_cooldown(slotbar_index, slot_index)
                    .unwrap_or(100)
                    .try_into();
                if last_time.is_some() && cooldown.is_ok() {
                    let slot_last_time = last_time.unwrap().elapsed().as_millis();
                    if slot_last_time > cooldown.unwrap() {
                        self.slots_usage_last_time[slotbar_index][slot_index] = None;
                    }
                }
                slot_index += 1;
            }
            slotbar_index += 1;
            drop(slot_index);
        }
        drop(slotbar_index);
    }

    fn get_slot_for(
        &mut self,
        config: &SupportConfig,
        threshold: Option<u32>,
        slot_type: SlotType,
        send: bool,
    ) -> Option<(usize, usize)> {
        if let Some(slot_index) =
            config.get_usable_slot_index(slot_type, threshold, self.slots_usage_last_time)
        {
            if send {
                //slog::debug!(self.logger, "Slot usage"; "slot_type" => slot_type.to_string(), "value" => threshold);
                self.send_slot(slot_index);
            }

            return Some(slot_index);
        }
        return None;
    }

    fn send_slot(&mut self, slot_index: (usize, usize)) {
        // Send keystroke for first slot mapped to pill
        send_slot_eval(self.window, slot_index.0, slot_index.1);
        // Update usage last time
        self.slots_usage_last_time[slot_index.0][slot_index.1] = Some(Instant::now());
    }

    fn check_buffs(&mut self, config: &SupportConfig) {
        if self.last_buff_usage.elapsed().as_millis() > 2000 {
            self.last_buff_usage = Instant::now();
            self.get_slot_for(config, None, SlotType::BuffSkill, true);
        }
    }

    fn check_restorations(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {
        // Check HP
        let stat = Some(image.client_stats.hp.value);
        if image.client_stats.hp.value > 0 {
            if self
                .get_slot_for(config, stat, SlotType::Pill, true)
                .is_none()
            {
                self.get_slot_for(config, stat, SlotType::Food, true);
            }
        }

        //Check target HP
        let stat = Some(image.client_stats.target_hp.value);
        if image.client_stats.target_hp.value > 0 {
            self.get_slot_for(config, stat, SlotType::HealSkill, true);
        }

        // Check MP
        let stat = Some(image.client_stats.mp.value);
        if image.client_stats.mp.value > 0 {
            self.get_slot_for(config, stat, SlotType::MpRestorer, true);
        }

        // Check FP
        let stat = Some(image.client_stats.fp.value);
        if image.client_stats.fp.value > 0 {
            self.get_slot_for(config, stat, SlotType::FpRestorer, true);
        }
    }
}
