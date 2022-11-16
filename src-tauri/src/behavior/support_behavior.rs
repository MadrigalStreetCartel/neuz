use std::time::{Instant, Duration};

use slog::Logger;
use tauri::Window;

use crate::{
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FrontendInfo, SlotType, SupportConfig},
    movement::MovementAccessor,
    play,
};

use super::{Behavior, SlotsUsage};

pub struct SupportBehavior<'a> {
    movement: &'a MovementAccessor,
    slots_usage: SlotsUsage,
    avoid_obstacle_direction: String,
    last_far_from_target: Option<Instant>,
    //is_on_flight: bool,
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(_logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            movement,
            avoid_obstacle_direction: "D".to_owned(),
            last_far_from_target: None,
            slots_usage: SlotsUsage::new(window.clone(), "Support".to_string()),
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
        let config = bot_config.support_config();
        let target_marker = image.identify_target_marker(true);
        self.slots_usage.update_slots_usage();

        if image.client_stats.target_hp.value == 0 && target_marker.is_some() {
            self.slots_usage.get_slot_for(None, SlotType::RezSkill, true);
            self.slots_usage.reset_slots_usage();
            return;
        }

        self.slots_usage.check_restorations(image);
        //std::thread::sleep(Duration::from_millis(100));

        // Send chat message if there's
        self.slots_usage.get_slot_for(None, SlotType::ChatMessage, true);

        if image.client_stats.target_hp.value > 0 {
            if let Some(target_marker) = target_marker {
                let marker_distance = image.get_target_marker_distance(target_marker);
                if marker_distance > 200 {
                    if self.last_far_from_target.is_none() {
                        self.last_far_from_target = Some(Instant::now());
                    }
                    self.avoid_obstacle(bot_config, config);
                } else {
                    self.last_far_from_target = None;
                    self.slots_usage.check_buffs();
                }
            } else {
                self.avoid_obstacle(bot_config, config);
            }
        }
    }
}

impl<'a> SupportBehavior<'_> {

    fn avoid_obstacle(&mut self, bot_config: &BotConfig, config: &SupportConfig) {
        if let Some(last_far_from_target) = self.last_far_from_target {
            if last_far_from_target.elapsed().as_millis() > bot_config.obstacle_avoidance_cooldown() {
                    self.move_circle_pattern();
            }
        } else{
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
}
