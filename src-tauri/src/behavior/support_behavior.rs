use std::time::{Duration, Instant};

use slog::Logger;
use tauri::Window;

use super::Behavior;
use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, SlotType, SupportConfig},
    movement::MovementAccessor,
    platform::send_slot_eval,
    play,
};

pub struct SupportBehavior<'a> {
    movement: &'a MovementAccessor,
    window: &'a Window,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_buff_usage: Instant,
    last_jump_time: Instant,
    avoid_obstacle_direction: String,
    last_far_from_target: Option<Instant>,
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
            avoid_obstacle_direction: "D".to_owned(),
            last_far_from_target: None,
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
        let target_marker = image.identify_target_marker(true);
        self.update_slots_usage(config);

        if image.client_stats.target_hp.value == 0 && target_marker.is_some() {
            self.get_slot_for(config, None, SlotType::RezSkill, true);
            self.slots_usage_last_time = [[None; 10]; 9];
            return;
        }

        use crate::movement::prelude::*;
        PressKey("C");


        self.check_restorations(config, image);
        std::thread::sleep(Duration::from_millis(100));

        if image.client_stats.target_hp.value > 0 {
            if let Some(target_marker) = target_marker {
                let marker_distance = image.get_target_marker_distance(target_marker);
                if marker_distance > 200 {
                    if self.last_far_from_target.is_none() {
                        self.last_far_from_target = Some(Instant::now());
                    }
                    self.avoid_obstacle(config);
                } else {
                    self.last_far_from_target = None;
                    self.check_buffs(config, image);
                }
            } else {
                self.avoid_obstacle(config);
            }
        }
    }
}

impl SupportBehavior<'_> {
    fn avoid_obstacle(&mut self, config: &SupportConfig) {
        if let Some(last_far_from_target) = self.last_far_from_target {
            if last_far_from_target.elapsed().as_millis() > config.obstacle_avoidance_cooldown() {
                self.move_circle_pattern();
            }
        } else {
            use crate::movement::prelude::*;
            play!(self.movement => [
                PressKey("Z"),
            ]);
        }
    }

    fn move_circle_pattern(&mut self) {
        // low rotation duration means big circle, high means little circle
        use crate::movement::prelude::*;
        play!(self.movement => [
            HoldKeys(vec!["W", "Space", &self.avoid_obstacle_direction]),
            Wait(dur::Fixed(200)),
            ReleaseKey(&self.avoid_obstacle_direction),
            Wait(dur::Fixed(500)),
            ReleaseKeys(vec!["Space", "W"]),
            HoldKeyFor("S", dur::Fixed(50)),
            PressKey("Z"),
            Wait(dur::Fixed(300)),
        ]);

        self.avoid_obstacle_direction = {
            if self.avoid_obstacle_direction == "D" {
                "A".to_owned()
            } else {
                "D".to_owned()
            }
        }
    }

    /// Update slots cooldown timers
    fn update_slots_usage(&mut self, config: &SupportConfig) {
        for (slotbar_index, slot_bars) in self.slots_usage_last_time.into_iter().enumerate() {
            for (slot_index, last_time) in slot_bars.into_iter().enumerate() {
                let cooldown = config
                    .get_slot_cooldown(slotbar_index, slot_index)
                    .unwrap_or(100)
                    .try_into();
                if let Some(last_time) = last_time {
                    if let Ok(cooldown) = cooldown {
                        let slot_last_time = last_time.elapsed().as_millis();
                        if slot_last_time > cooldown {
                            self.slots_usage_last_time[slotbar_index][slot_index] = None;
                        }
                    }
                }
            }
        }
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
                if slot_type == SlotType::HealSkill {
                    // Set last buff usage to now so that a buff doesn't try and happen immediately after a heal
                    self.last_buff_usage = Instant::now();
                }
            }

            return Some(slot_index);
        }
        None
    }

    fn send_slot(&mut self, slot_index: (usize, usize)) {
        // Send keystroke for first slot mapped to pill
        send_slot_eval(self.window, slot_index.0, slot_index.1);
        // Update usage last time
        self.slots_usage_last_time[slot_index.0][slot_index.1] = Some(Instant::now());
    }

    fn check_buffs(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {
        if self.last_buff_usage.elapsed().as_millis() > config.interval_between_buffs() {
            self.last_buff_usage = Instant::now();

            self.full_buffing(config,image);
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    fn full_buffing(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer, ) {
        let all_buffs= config.get_all_usable_slot_for_type_index(SlotType::BuffSkill);
        for slot_index in all_buffs {
            std::thread::sleep(Duration::from_millis(1000));
            self.send_slot(slot_index);
        }


        self.check_restorations(config, image);

    }
    fn check_restorations(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {

        // Checking our own stuff first to keep alive
        let health_stat = Some(image.client_stats.hp.value);
        if image.client_stats.hp.value > 0 {
            // Use a HealSkill if configured when health is under 85

            if  image.client_stats.hp.value < 20 {
                // Take a pill if health is less than 40, ideally should not be often
                self.get_slot_for(config, health_stat, SlotType::Pill, true);
            }
            if image.client_stats.hp.value < 85 {
                self.get_slot_for(config, health_stat, SlotType::HealSkill, true);
            }

            if image.client_stats.hp.value < 60 {
                // Eat food if health under 70
                self.get_slot_for(config, health_stat, SlotType::Food, true);
            }

            if image.client_stats.hp.value < 50 {
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
                std::thread::sleep(Duration::from_millis(100));
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
                std::thread::sleep(Duration::from_millis(100));
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
            }

        }

        // Check MP
        let mp_stat = Some(image.client_stats.mp.value);
        if image.client_stats.mp.value > 0 &&  image.client_stats.mp.value < 60 {
            self.get_slot_for(config, mp_stat, SlotType::MpRestorer, true);
        }

        // Check FP
        let fp_stat = Some(image.client_stats.fp.value);
        if image.client_stats.fp.value > 0 &&  image.client_stats.mp.value < 60 {
            self.get_slot_for(config, fp_stat, SlotType::FpRestorer, true);
        }

        //Check target HP
        let target_hp = Some(image.client_stats.target_hp.value);
        if image.client_stats.target_hp.value > 0 && image.client_stats.target_hp.value < 85{
            self.get_slot_for(config, target_hp, SlotType::HealSkill, true);
        }


    }
}
