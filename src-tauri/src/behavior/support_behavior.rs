use std::time::{Duration, Instant};

use libscreenshot::shared::Area;
use rand::prelude::SliceRandom;
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{Bounds, MobType, PixelDetection, PixelDetectionKind, Target, TargetType},
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, SupportConfig, FrontendInfo, SlotType},
    movement::MovementAccessor,
    platform::{send_slot, Key, PlatformAccessor},
    play,
    utils::DateTime,
};

use super::Behavior;

pub struct SupportBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    platform: &'a PlatformAccessor<'a>,
    movement: &'a MovementAccessor,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement: &'a MovementAccessor,
    ) -> Self {
        Self {
            rng: rand::thread_rng(),
            logger,
            platform,
            movement,
            slots_usage_last_time: [[None; 10]; 9],
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer,
    ) {
        let config = config.support_config();

       /*  // Update all needed timestamps
        self.update_timestamps(config);

        // Check whether something should be restored
        self.check_restorations(config, image);

        // Use buffs Yiha
        self.check_buffs(config); */
        self.update_slots_usage(config);
    }


}

impl<'a> SupportBehavior<'_> {
    fn update_slots_usage(&mut self, config: &SupportConfig) {
        // Update slots cooldown timers
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
        if let Some(slot_index) = config.get_usable_slot_index(
            slot_type,
            &mut self.rng,
            threshold,
            self.slots_usage_last_time,
        ) {
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
        send_slot(slot_index.0, slot_index.1.into());

        // Update usage last time
        self.slots_usage_last_time[slot_index.0][slot_index.1] = Some(Instant::now());
    }

}
