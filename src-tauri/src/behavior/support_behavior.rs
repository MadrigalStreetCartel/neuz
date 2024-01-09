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
    logger: &'a Logger,
    movement: &'a MovementAccessor,
    window: &'a Window,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_buff_usage: Instant,
    last_jump_time: Instant,
    last_action_time: Instant,
    avoid_obstacle_direction: String,
    last_far_from_target: Option<Instant>,
    initial_full_buff: bool,
    //is_on_flight: bool,
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            logger,
            movement,
            window,
            slots_usage_last_time: [[None; 10]; 9],
            last_buff_usage: Instant::now(),
            last_jump_time: Instant::now(),
            last_action_time: Instant::now(),
            avoid_obstacle_direction: "D".to_owned(),
            last_far_from_target: None,
            initial_full_buff: true,
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


        //add movement every minute to try to avoid bot detection
        if self.last_jump_time.elapsed().as_millis() > 10000 {
            use crate::movement::prelude::*;
            play!(self.movement => [
                // Rotate in random direction for a random duration
                Rotate(rot::Right, dur::Fixed(50)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Fixed(50)),
            ]);

            self.last_jump_time = Instant::now();
        }



        //Res the target if it dies
        if image.client_stats.target_hp.value == 0 && target_marker.is_some() {
            self.get_slot_for(config, None, SlotType::RezSkill, true);
            self.slots_usage_last_time = [[None; 10]; 9];
            return;
        }

        if self.initial_full_buff {
            self.full_buffing(config, image);
            self.initial_full_buff = false;
            self.last_buff_usage = Instant::now();
            std::thread::sleep(Duration::from_millis(100));
        }

        //check if we have a valid target and if not, check the AFK time to dc
        if config.on_afk_disconnect() && target_marker.is_none() &&
            self.last_action_time.elapsed().as_millis() > config.afk_timeout() {
            _frontend_info.set_afk_ready_to_disconnect(true);
        }

        //This is where we heal the target
        self.check_restorations(config, image);
        self.use_party_skills(config);

        //Run the buffs that are available
        self.get_slot_for(config, None, SlotType::BuffSkill, true);
        std::thread::sleep(Duration::from_millis(1500));

        //do a full buff
        // if self.last_buff_usage.elapsed().as_millis() > config.interval_between_buffs() {
        //     self.full_buffing(config, image);
        //     self.last_buff_usage = Instant::now();
        //     self.last_action_time = Instant::now();
        // }


        //detect distance to target and avoid obstacle if needed
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


    fn full_buffing(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {

        let all_buffs = config.get_all_usable_slot_for_type(SlotType::BuffSkill, [[None; 10]; 9]);

        for slot_index in all_buffs {
            self.send_slot(slot_index);
            std::thread::sleep(Duration::from_millis(1500));

            self.get_slot_for(config, None, SlotType::HealSkill, true);

            std::thread::sleep(Duration::from_millis(500));
            self.get_slot_for(config, None, SlotType::AOEHealSkill, true);
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    fn use_party_skills(&mut self, config: &SupportConfig) {
        let party_skills = config.get_all_usable_slot_for_type(SlotType::PartySkill, self.slots_usage_last_time);
        for slot_index in party_skills {
            self.send_slot(slot_index);
        }
    }

    fn check_restorations(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {

        //Check target HP
        let target_hp = Some(image.client_stats.target_hp.value);
        if image.client_stats.target_hp.value > 0 {
            if image.client_stats.target_hp.value < 85 {
                self.get_slot_for(config, target_hp, SlotType::HealSkill, true);
                std::thread::sleep(Duration::from_millis(500));

                if image.client_stats.target_hp.value < 60 {
                    self.get_slot_for(config, target_hp, SlotType::AOEHealSkill, true);
                }
            }
        }
        // Checking our own stuff first to keep alive
        let health_stat = Some(image.client_stats.hp.value);
        if image.client_stats.hp.value > 0 {
            // Use a HealSkill if configured when health is under 85

            if image.client_stats.hp.value < 20 {
                // Take a pill if health is less than 40, ideally should not be often
                self.get_slot_for(config, health_stat, SlotType::Pill, true);
            }

            if image.client_stats.hp.value < 95 {
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
            }

            if image.client_stats.hp.value < 60 {
                // Eat food if health under 70
                self.get_slot_for(config, health_stat, SlotType::Food, true);
            }
        }

        // Check MP
        let mp_stat = Some(image.client_stats.mp.value);
        if image.client_stats.mp.value > 0 && image.client_stats.mp.value < 60 {
            self.get_slot_for(config, mp_stat, SlotType::MpRestorer, true);
        }

        // Check FP
        let fp_stat = Some(image.client_stats.fp.value);
        if image.client_stats.fp.value > 0 && image.client_stats.mp.value < 60 {
            self.get_slot_for(config, fp_stat, SlotType::FpRestorer, true);
        }
    }
}
