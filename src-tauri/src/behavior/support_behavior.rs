use std::time::{Instant, Duration};

use slog::Logger;
use tauri::Window;

use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, SlotType},
    movement::MovementAccessor,
    play, platform::KeyManager,
};

use super::{Behavior, SlotsUsage};

pub struct SupportBehavior<'a> {
    movement: &'a MovementAccessor,
    slots_usage: SlotsUsage<'a>,
    key_manager: &'a KeyManager,
    avoid_obstacle_direction: String,
    last_far_from_target: Option<Instant>,
    target_die_time: Option<Instant>,
    //is_on_flight: bool,
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(_logger: &'a Logger, movement: &'a MovementAccessor, key_manager: &'a KeyManager) -> Self {
        Self {
            movement,
            avoid_obstacle_direction: "D".to_owned(),
            last_far_from_target: None,
            slots_usage: SlotsUsage::new(key_manager, "Support".to_string()),
            key_manager: key_manager,
            target_die_time: None,
            //is_on_flight: false,
        }
    }

    fn start(&mut self, config: &BotConfig) {
        self.slots_usage.update_config(config.clone());
    }

    fn update(&mut self, config: &BotConfig) {
        self.slots_usage.update_config(config.clone());
    }

    fn stop(&mut self, _config: &BotConfig) {
        self.slots_usage.reset_slots_usage();
    }

    fn run_iteration(
        &mut self,
        _frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer,
    ) {
        let bot_config = config;
        //let config = bot_config.support_config();
        let target_marker = image.client_stats.target_marker;
        self.slots_usage.update_slots_usage();

        if image.client_stats.target_hp.value == 0 && target_marker.is_some() {
            if self.target_die_time.is_none(){
                self.target_die_time = Some(Instant::now());
            } else if self.target_die_time.unwrap().elapsed().as_millis() > 1000 {
                self.avoid_obstacle(bot_config);
            }
            self.slots_usage.get_slot_for(None, SlotType::RezSkill, true);
            return;
        }else if image.client_stats.target_hp.value > 0 && self.target_die_time.is_some() {
            self.slots_usage.reset_slots_usage();
            self.target_die_time = None;
            if image.client_stats.target_hp.value != 100 {
                self.slots_usage.get_slot_for(None, SlotType::HealSkill, true);
            }
        }

        self.slots_usage.check_restorations(image);
        std::thread::sleep(Duration::from_millis(100));

        // Send chat message if there's
        self.slots_usage.get_slot_for(None, SlotType::ChatMessage, true);

        if image.client_stats.target_hp.value > 0 {
            if let Some(target_marker) = target_marker {
                let marker_distance = target_marker.get_distance();
                if marker_distance > 150 {
                    if self.last_far_from_target.is_none() {
                        self.last_far_from_target = Some(Instant::now());
                    }
                    if let Some(last_far_from_target) = self.last_far_from_target {
                        if last_far_from_target.elapsed().as_millis() > bot_config.obstacle_avoidance_cooldown() {
                            self.avoid_obstacle(bot_config);
                        }
                    }
                } else {
                    self.last_far_from_target = None;
                    use crate::movement::prelude::*;
                    play!(self.movement => [
                        PressKey("Z"),
                    ]);
                    self.slots_usage.check_buffs();
                }
            } else {
                self.avoid_obstacle(bot_config);
            }
        }
    }
}

impl<'a> SupportBehavior<'_> {

    fn avoid_obstacle(&mut self, bot_config: &BotConfig) {
        self.move_circle_pattern();
        use crate::movement::prelude::*;
        play!(self.movement => [
            PressKey("Z"),
        ]);
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
}
