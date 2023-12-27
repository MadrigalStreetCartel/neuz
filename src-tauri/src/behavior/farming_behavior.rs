use std::time::{Duration, Instant};

use rand::prelude::SliceRandom;
use slog::Logger;
use tauri::{Manager, Window};

use super::Behavior;
use crate::{
    data::{Bounds, MobType, Point, Target, TargetType},
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FarmingConfig, FrontendInfo, SlotType},
    movement::MovementAccessor,
    platform::{eval_mob_click, send_slot_eval},
    play,
    utils::DateTime,
};

#[derive(Debug, Clone, Copy)]
enum State {
    Buffing,
    NoEnemyFound,
    SearchingForEnemy,
    EnemyFound(Target),
    Attacking(Target),
    AfterEnemyKill(Target),
}

pub struct FarmingBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    movement: &'a MovementAccessor,
    window: &'a Window,
    state: State,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_initial_attack_time: Instant,
    last_kill_time: Instant,
    avoided_bounds: Vec<(Bounds, Instant, u128)>,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
    obstacle_avoidance_count: u32,
    last_summon_pet_time: Option<Instant>,
    last_killed_type: MobType,
    start_time: Instant,
    already_attack_count: u32,
    last_buff_usage: Instant,
    last_click_pos: Option<Point>,
    stealed_target_count: u32,
    last_no_ennemy_time: Option<Instant>,
    concurrent_mobs_under_attack: u32,
}

impl<'a> Behavior<'a> for FarmingBehavior<'a> {
    fn new(logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            logger,
            movement,
            window,
            rng: rand::thread_rng(),
            state: State::Buffing, //Start with buff before attacking
            slots_usage_last_time: [[None; 10]; 9],
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            avoided_bounds: vec![],
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
            obstacle_avoidance_count: 0,
            last_summon_pet_time: None,
            last_killed_type: MobType::Passive,
            start_time: Instant::now(),
            already_attack_count: 0,
            last_buff_usage: Instant::now(),
            last_click_pos: None,
            stealed_target_count: 0,
            last_no_ennemy_time: None,
            concurrent_mobs_under_attack: 0,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {
        self.slots_usage_last_time = [[None; 10]; 9];
    }

    fn run_iteration(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer,
    ) {
        let config = config.farming_config();
        // Update all needed timestamps
        self.update_timestamps(config);

        // Check whether something should be restored
        self.check_restorations(config, image);

        // Check state machine
        self.state = match self.state {
            State::Buffing => self.full_buffing(config, image),
            State::NoEnemyFound => self.on_no_enemy_found(config),
            State::SearchingForEnemy => self.on_searching_for_enemy(config, image),
            State::EnemyFound(mob) => self.on_enemy_found(mob),
            State::Attacking(mob) => self.on_attacking(config, mob, image),
            State::AfterEnemyKill(_) => self.after_enemy_kill(frontend_info, config),
        };

        frontend_info.set_is_attacking(self.is_attacking);
    }
}

impl FarmingBehavior<'_> {
    fn update_timestamps(&mut self, config: &FarmingConfig) {
        self.update_pickup_pet(config);

        self.update_slots_usage(config);

        self.update_avoid_bounds();
    }

    /// Update avoid bounds cooldowns timers
    fn update_avoid_bounds(&mut self) {
        let mut result: Vec<(Bounds, Instant, u128)> = vec![];
        for n in 0..self.avoided_bounds.len() {
            let current = self.avoided_bounds[n];
            if current.1.elapsed().as_millis() < current.2 {
                result.push(current);
            }
        }
        self.avoided_bounds = result;
    }

    /// Check whether pickup pet should be unsummoned
    fn update_pickup_pet(&mut self, config: &FarmingConfig) {
        if let Some(pickup_pet_slot_index) = config.slot_index(SlotType::PickupPet) {
            if let Some(last_time) = self.last_summon_pet_time {
                if last_time.elapsed().as_millis()
                    > config
                    .get_slot_cooldown(pickup_pet_slot_index.0, pickup_pet_slot_index.1)
                    .unwrap_or(3000) as u128
                {
                    send_slot_eval(
                        self.window,
                        pickup_pet_slot_index.0,
                        pickup_pet_slot_index.1,
                    );
                    self.last_summon_pet_time = None;
                }
            }
        }
    }

    /// Update slots cooldown timers
    fn update_slots_usage(&mut self, config: &FarmingConfig) {
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
        config: &FarmingConfig,
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

    /// Pickup items on the ground.
    fn pickup_items(&mut self, config: &FarmingConfig) {
        let slot = self.get_slot_for(config, None, SlotType::PickupPet, false);
        if let Some(index) = slot {
            if self.last_summon_pet_time.is_none() {
                send_slot_eval(self.window, index.0, index.1);
                self.last_summon_pet_time = Some(Instant::now());
            } else {
                // if pet is already out, just reset it's timer
                self.last_summon_pet_time = Some(Instant::now());
            }
        } else {
            let slot = self.get_slot_for(config, None, SlotType::PickupMotion, false);
            if let Some(index) = slot {
                for _i in 1..7 {
                    send_slot_eval(self.window, index.0, index.1);
                }
            }
        }
    }

    fn check_restorations(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer) {
        // Check HP
        let health_stat = Some(image.client_stats.hp.value);
        if image.client_stats.hp.value > 0 {
            // Use a HealSkill if configured when health is under 85

            if image.client_stats.hp.value < 20 {
                // Take a pill if health is less than 40, ideally should not be often
                self.get_slot_for(config, health_stat, SlotType::Pill, true);
            }

            if image.client_stats.hp.value < 70 {
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
                std::thread::sleep(Duration::from_millis(100));
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
                std::thread::sleep(Duration::from_millis(100));
                self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true);
            }

            if image.client_stats.hp.value < 85 {
                self.get_slot_for(config, health_stat, SlotType::HealSkill, true);
            }

            if image.client_stats.hp.value < 60 {
                // Eat food if health under 70
                self.get_slot_for(config, health_stat, SlotType::Food, true);
            }

            // Check MP
            let mp_stat = Some(image.client_stats.mp.value);
            if image.client_stats.mp.value > 0 && image.client_stats.mp.value < 50 {
                self.get_slot_for(config, mp_stat, SlotType::MpRestorer, true);
            }

            // Check FP

            let fp_stat = Some(image.client_stats.fp.value);
            if image.client_stats.fp.value > 0 && image.client_stats.fp.value  < 50 {
                self.get_slot_for(config, fp_stat, SlotType::FpRestorer, true);
            }
        }
    }

    fn full_buffing(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer) -> State {
        let all_buffs = config.get_all_usable_slot_for_type_index(SlotType::BuffSkill);

        for slot_index in all_buffs {
            std::thread::sleep(Duration::from_millis(2000));
            self.send_slot(slot_index);
        }
        self.check_restorations(config, image);

        // Transition to next state
        return State::NoEnemyFound;
    }

    fn check_buffs(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer) {
        if self.last_buff_usage.elapsed().as_millis() > config.interval_between_buffs() {
            self.full_buffing(config, image);
            self.last_buff_usage = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    fn on_no_enemy_found(&mut self, config: &FarmingConfig) -> State {
        if let Some(last_no_ennemy_time) = self.last_no_ennemy_time {
            if config.mobs_timeout() > 0
                && last_no_ennemy_time.elapsed().as_millis() > config.mobs_timeout()
            {
                self.window.app_handle().exit(0);
            }
        } else {
            self.last_no_ennemy_time = Some(Instant::now());
        }
        use crate::movement::prelude::*;
        // Try rotating first in order to locate nearby enemies
        if self.rotation_movement_tries < 30 {
            play!(self.movement => [
                // Rotate in random direction for a random duration
                Rotate(rot::Right, dur::Fixed(50)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Fixed(50)),
            ]);
            self.rotation_movement_tries += 1;

            // Transition to next state
            return State::SearchingForEnemy;
        }

        // Check whether bot should stay in area
        let circle_pattern_rotation_duration = config.circle_pattern_rotation_duration();
        if circle_pattern_rotation_duration > 0 {
            self.move_circle_pattern(circle_pattern_rotation_duration);
        } else {
            self.rotation_movement_tries = 0;
            return self.state;
        }
        // Transition to next state
        State::SearchingForEnemy
    }

    fn move_circle_pattern(&self, rotation_duration: u64) {
        // low rotation duration means big circle, high means little circle
        use crate::movement::prelude::*;
        play!(self.movement => [
            HoldKeys(vec!["W", "Space", "D"]),
            Wait(dur::Fixed(rotation_duration)),
            ReleaseKey("D"),
            Wait(dur::Fixed(20)),
            ReleaseKeys(vec!["Space", "W"]),
            HoldKeyFor("S", dur::Fixed(50)),
        ]);
    }

    fn on_searching_for_enemy(
        &mut self,
        config: &FarmingConfig,
        image: &mut ImageAnalyzer,
    ) -> State {
        slog::debug!(self.logger, "On searching for enemy: ");
        //if a mob is attacking us while we search
        self.check_restorations(config, image);

        if config.is_stop_fighting() {
            return State::Attacking(Target::default());
        }
        let mobs = image.identify_mobs(config);
        if mobs.is_empty() {
            // Transition to next state
            State::NoEnemyFound
        } else {
            // Calculate max distance of mobs
            let max_distance = match config.circle_pattern_rotation_duration() == 0 {
                true => 325,
                false => 1000,
            };

            let mob_list = self.get_list_of_mobs(config, image, mobs);


            // inverted conditionals to make it easier to read
            // Check again if we have a list of mobs
            if mob_list.is_empty() {
                // Transition to next state
                State::NoEnemyFound
            } else {
                self.rotation_movement_tries = 0;
                //slog::debug!(self.logger, "Found mobs"; "mob_type" => mob_type, "mob_count" => mob_list.len());
                if let Some(mob) = {
                    // Try avoiding detection of last killed mob
                    if !self.avoided_bounds.is_empty() {
                        image.find_closest_mob(mob_list.as_slice(), Some(&self.avoided_bounds), max_distance, self.logger)
                    } else {
                        image.find_closest_mob(mob_list.as_slice(), None, max_distance, self.logger)
                    }
                } {
                    // Transition to next state
                    State::EnemyFound(*mob)
                } else {
                    // Transition to next state
                    State::SearchingForEnemy
                }
            }
        }
    }

    fn get_list_of_mobs(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer, mobs: Vec<Target>) -> Vec<Target> {
        let mut mob_list = Vec::new();

        // Get aggressive mobs to prioritize them
        if config.prioritize_aggro() {
            mob_list = mobs
                .iter()
                .filter(|m| m.target_type == TargetType::Mob(MobType::Aggressive))
                .cloned()
                .collect::<Vec<_>>();

            // Check if there's aggressive mobs otherwise collect passive mobs
            if (mob_list.is_empty()
                || self.last_killed_type == MobType::Aggressive
                && mob_list.len() == 1
                && self.last_kill_time.elapsed().as_millis() < 5000)
                && image.client_stats.hp.value >= config.min_hp_attack()
            {
                mob_list = mobs
                    .iter()
                    .filter(|m| m.target_type == TargetType::Mob(MobType::Passive))
                    .cloned()
                    .collect::<Vec<_>>();
            }
        } else {
            mob_list = mobs
                .iter()
                .cloned()
                .collect::<Vec<_>>();
        }

        mob_list
    }

    fn avoid_last_click(&mut self) {
        if let Some(point) = self.last_click_pos {
            let marker = Bounds::new(point.x - 1, point.y - 1, 2, 2);
            self.avoided_bounds.push((marker, Instant::now(), 5000));
        }
    }

    fn on_enemy_found(&mut self, mob: Target) -> State {
        // Transform attack coords into local window coords
        let point = mob.get_attack_coords();

        self.last_click_pos = Some(point);

        // Set cursor position and simulate a click
        eval_mob_click(self.window, point);

        // Wait a few ms before transitioning state
        std::thread::sleep(Duration::from_millis(500));
        State::Attacking(mob)
    }

    fn abort_attack(&mut self, image: &mut ImageAnalyzer) -> State {
        use crate::movement::prelude::*;
        self.is_attacking = false;
        if self.already_attack_count > 0 {
            // Target marker found
            if let Some(marker) = image.identify_target_marker(false) {
                self.avoided_bounds.push((
                    marker.bounds.grow_by(self.already_attack_count * 10),
                    Instant::now(),
                    2000,
                ));
                self.already_attack_count += 1;
            }
        } else {
            self.obstacle_avoidance_count = 0;
            use crate::movement::prelude::*;
            self.is_attacking = false;
            self.avoid_last_click();
        }
        play!(self.movement => [
            PressKey("Escape"),
        ]);
        State::SearchingForEnemy
    }

    fn avoid_obstacle_no_jump(&mut self, image: &mut ImageAnalyzer, max_avoid: u32) -> bool {
        if self.obstacle_avoidance_count < max_avoid {
            use crate::movement::prelude::*;
            if self.obstacle_avoidance_count == 0 {
                play!(self.movement => [
                    PressKey("Z"),
                    HoldKeys(vec!["W"]),
                    Wait(dur::Fixed(800)),
                    ReleaseKeys(vec![ "W"]),
                ]);
            } else {
                let rotation_key = ["A", "D"].choose(&mut self.rng).unwrap_or(&"A");
                // Move into a random direction while jumping
                play!(self.movement => [
                    HoldKeys(vec!["W"]),
                    HoldKeyFor(rotation_key, dur::Fixed(200)),
                    Wait(dur::Fixed(800)),
                    ReleaseKeys(vec!["W"]),
                    PressKey("Z"),
                ]);
            }
            image.client_stats.target_hp.reset_last_update_time();
            self.obstacle_avoidance_count += 1;
            false
        } else {
            self.abort_attack(image);
            true
        }
    }
    fn avoid_obstacle(&mut self, image: &mut ImageAnalyzer, max_avoid: u32) -> bool {
        if self.obstacle_avoidance_count < max_avoid {
            use crate::movement::prelude::*;
            if self.obstacle_avoidance_count == 0 {
                play!(self.movement => [
                    PressKey("Z"),
                    HoldKeys(vec!["W", "Space"]),
                    Wait(dur::Fixed(800)),
                    ReleaseKeys(vec!["Space", "W"]),
                ]);
            } else {
                let rotation_key = ["A", "D"].choose(&mut self.rng).unwrap_or(&"A");
                // Move into a random direction while jumping
                play!(self.movement => [
                    HoldKeys(vec!["W", "Space"]),
                    HoldKeyFor(rotation_key, dur::Fixed(200)),
                    Wait(dur::Fixed(800)),
                    ReleaseKeys(vec!["Space", "W"]),
                    PressKey("Z"),
                ]);
            }

            image.client_stats.target_hp.reset_last_update_time();
            self.obstacle_avoidance_count += 1;
            false
        } else {
            self.abort_attack(image);
            true
        }
    }

    fn on_attacking(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &mut ImageAnalyzer,
    ) -> State {
        //Adding restoration checks in case health is low
        self.check_restorations(config, image);


        let is_npc =
            image.client_stats.target_hp.value == 100 && image.client_stats.target_mp.value == 0;
        let is_mob =
            image.client_stats.target_hp.value > 0 && image.client_stats.target_mp.value > 0;
        let is_mob_alive = image.identify_target_marker(false).is_some()
            || image.client_stats.target_mp.value > 0
            || image.client_stats.target_hp.value > 0;

        // slog::debug!(self.logger, " On main attacking: "; "self.is_attacking"=> self.is_attacking, "config.is_stop_fighting()"=>config.is_stop_fighting(), "is_mob_alive"=>is_mob_alive,
        // "is_npc"=>is_npc,"is_mob"=>is_mob);


        if !self.is_attacking && !config.is_stop_fighting() {
            if is_npc {
                self.avoid_last_click();
                return State::SearchingForEnemy;
            } else if is_mob {
                self.rotation_movement_tries = 0;
                let hp_last_update = image.client_stats.hp.last_update_time.unwrap();
                // Detect if mob was attacked

                if image.client_stats.target_hp.value < 100 && config.prevent_already_attacked() {
                    // // If we didn't took any damages abort attack
                    if hp_last_update.elapsed().as_millis() > 5000 {
                        return self.abort_attack(image);
                    } else if self.stealed_target_count > 5 {
                        self.stealed_target_count = 0;
                        self.already_attack_count = 1;
                    }
                }
            } else {
                // Not a mob we go search for another
                self.avoid_last_click();
                return State::SearchingForEnemy;
            }
        } else if !self.is_attacking && config.is_stop_fighting() && !is_mob {
            return self.state;
        }

        if is_mob_alive {
            // Engaging combat
            if !self.is_attacking {
                self.obstacle_avoidance_count = 0;
                self.last_initial_attack_time = Instant::now();
                self.is_attacking = true;
                self.already_attack_count = 0;
            }

            let last_target_hp_update = image
                .client_stats
                .target_hp
                .last_update_time
                .unwrap()
                .elapsed()
                .as_millis();

            // Obstacle avoidance
            if image.identify_target_marker(false).is_none()
                || last_target_hp_update > config.obstacle_avoidance_cooldown()
            {
                if image.client_stats.target_hp.value == 100 {
                    if self.avoid_obstacle(image, 2) {
                        return State::SearchingForEnemy;
                    }
                } else if self.avoid_obstacle(image, config.obstacle_avoidance_max_try()) {
                    return State::SearchingForEnemy;
                }
            }

            // slog::debug!(self.logger, "on attacking: ";"last_target_hp_update"=>last_target_hp_update ,"config.obstacle_avoidance_cooldown()"=>config.obstacle_avoidance_cooldown(), "self.concurrent_mobs_under_attack" => self.concurrent_mobs_under_attack, "image.client_stats.target_hp.value" => image.client_stats.target_hp.value);
            //

            if (config.max_aoe_farming() > 1) {
                if self.concurrent_mobs_under_attack < config.max_aoe_farming() {
                    self.get_slot_for(config, None, SlotType::AttackSkill, true);
                    std::thread::sleep(Duration::from_millis(2000));
                    // self.concurrent_mobs_under_attack = self.concurrent_mobs_under_attack + 1;

                    // self.avoid_last_click();
                    // self.is_attacking = false;

                    // use crate::movement::prelude::*;
                    // play!(self.movement => [
                    //          PressKey("Escape"),
                    //      ]);
                    // play!(self.movement => [
                    //     // Rotate in random direction for a random duration
                    //     Rotate(rot::Right, dur::Fixed(3)),
                    //     // Wait a bit to wait for monsters to enter view
                    //     // Wait(dur::Fixed(3)),
                    // ]);
                    //
                    // return self.state;
                    // return self.abort_attack(image);


                    //inline abort attack
                    use crate::movement::prelude::*;
                    self.is_attacking = false;
                    if self.already_attack_count > 0 {
                        // Target marker found
                        if let Some(marker) = image.identify_target_marker(false) {
                            self.avoided_bounds.push((
                                marker.bounds.grow_by(self.already_attack_count * 10),
                                Instant::now(),
                                2000,
                            ));
                            self.already_attack_count += 1;
                        }
                    } else {
                        self.obstacle_avoidance_count = 0;
                        use crate::movement::prelude::*;
                        self.is_attacking = false;
                        self.avoid_last_click();
                    }
                    play!(self.movement => [
                        PressKey("Escape"),
                    ]);
                    self.concurrent_mobs_under_attack = self.concurrent_mobs_under_attack + 1;

                    return State::SearchingForEnemy

                    //inline abort attack


                    // self.avoid_obstacle(image, 1);

                    // play!(self.movement => [
                    //          PressKey("Escape"),
                    //      ]);
                    // play!(self.movement => [
                    //          PressKey("Escape"),
                    //       ]);
                    // slog::debug!(self.logger, "Inside the aoe search function, about to go to search for enemy");
                    // // return State::SearchingForEnemy;
                    // return self.abort_attack(image);
                } else {
                    self.get_slot_for(config, None, SlotType::AttackSkill, true);
                    std::thread::sleep(Duration::from_millis(1000));
                    self.get_slot_for(config, None, SlotType::AOEAttackSkill, true);
                    // std::thread::sleep(Duration::from_millis(1000));
                }
            } else {
                self.get_slot_for(config, None, SlotType::AttackSkill, true);
                // std::thread::sleep(Duration::from_millis(1000));
            }

            self.check_restorations(config, image);
            self.state
        } else if !is_mob_alive && image.client_stats.is_alive() && self.is_attacking {
            // Mob's dead
            match mob.target_type {
                TargetType::Mob(MobType::Aggressive) => self.last_killed_type = MobType::Aggressive,
                TargetType::Mob(MobType::Passive) => self.last_killed_type = MobType::Passive,
                TargetType::TargetMarker => {}
            }

            self.is_attacking = false;
            self.check_restorations(config, image);

            // Use buffs after we kill the mob so we don't buff mid fight
            self.check_buffs(config, image);
            self.concurrent_mobs_under_attack = 0;
            return State::AfterEnemyKill(mob);
        } else {
            self.is_attacking = false;
            return State::SearchingForEnemy;
        }
    }

    fn after_enemy_kill_debug(&mut self, frontend_info: &mut FrontendInfo) {

        // Let's introduce some stats
        let started_elapsed = self.start_time.elapsed();
        let started_formatted = DateTime::format_time(started_elapsed);

        let elapsed_time_to_kill = self.last_initial_attack_time.elapsed();
        let elapsed_search_time = self.last_kill_time.elapsed() - elapsed_time_to_kill;

        let search_time_as_secs = {
            if self.kill_count > 0 {
                elapsed_search_time.as_secs_f32()
            } else {
                elapsed_search_time.as_secs_f32() - started_elapsed.as_secs_f32()
            }
        };
        let time_to_kill_as_secs = elapsed_time_to_kill.as_secs_f32();

        let kill_per_minute =
            DateTime::format_float(60.0 / (time_to_kill_as_secs + search_time_as_secs), 0);
        let kill_per_hour = DateTime::format_float(kill_per_minute * 60.0, 0);

        let elapsed_search_time_string =
            format!("{}secs", DateTime::format_float(search_time_as_secs, 2));
        let elapsed_time_to_kill_string =
            format!("{}secs", DateTime::format_float(time_to_kill_as_secs, 2));

        let elapsed = format!(
            "Elapsed time : since start {} to kill {} to find {} ",
            started_formatted, elapsed_time_to_kill_string, elapsed_search_time_string
        );
        slog::debug!(self.logger, "Monster was killed {}", elapsed);

        frontend_info.set_kill_stats(
            (kill_per_minute, kill_per_hour),
            (
                elapsed_search_time.as_millis(),
                elapsed_time_to_kill.as_millis(),
            ),
        )
    }

    fn after_enemy_kill(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &FarmingConfig,
    ) -> State {
        self.kill_count += 1;
        frontend_info.set_kill_count(self.kill_count);
        self.after_enemy_kill_debug(frontend_info);

        self.stealed_target_count = 0;
        self.last_kill_time = Instant::now();

        // Pickup items
        self.pickup_items(config);
        self.concurrent_mobs_under_attack = 0;
        // Transition state
        State::SearchingForEnemy
    }
}
