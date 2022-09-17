use std::time::{Duration, Instant};

use libscreenshot::shared::Area;
use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{
        Bounds, MobType, PixelDetection, PixelDetectionKind, StatusBarKind, Target, TargetType,
    },
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FarmingConfig, SlotType},
    movement::MovementAccessor,
    platform::{send_keystroke, send_slot, Key, KeyMode, PlatformAccessor},
    play,
};

use super::Behavior;

#[derive(Debug, Clone, Copy)]
enum State {
    NoEnemyFound,
    SearchingForEnemy,
    EnemyFound(Target),
    Attacking(Target),
    AfterEnemyKill(Target),
}

pub struct FarmingBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    platform: &'a PlatformAccessor<'a>,
    movement: &'a MovementAccessor,
    state: State,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_initial_attack_time: Instant,
    last_kill_time: Instant,
    avoided_bounds: Vec<(Bounds, Instant, u128)>,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
    obstacle_avoidance_count: u32,
    missclick_count: u32,
    last_summon_pet_time: Option<Instant>,
}

impl<'a> Behavior<'a> for FarmingBehavior<'a> {
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement: &'a MovementAccessor,
    ) -> Self {
        Self {
            logger,
            platform,
            movement,
            rng: rand::thread_rng(),
            state: State::SearchingForEnemy,
            slots_usage_last_time: [[None; 10]; 9],
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            avoided_bounds: vec![],
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
            obstacle_avoidance_count: 0,
            missclick_count: 0,
            last_summon_pet_time: None,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(&mut self, config: &BotConfig, image: &mut ImageAnalyzer) {
        let config = config.farming_config();

        // Check whether pickup pet should be unsummoned
        if let Some(pickup_pet_slot_index) = config.get_slot_index(SlotType::PickupPet) {
            if let Some(last_time) = self.last_summon_pet_time {
                if last_time.elapsed().as_millis()
                    > config
                        .get_slot_cooldown(pickup_pet_slot_index.0, pickup_pet_slot_index.1)
                        .unwrap_or(3000) as u128
                {
                    send_slot(pickup_pet_slot_index.0, pickup_pet_slot_index.1.into());
                    self.last_summon_pet_time = None;
                }
            }
        }

        // Update slots cooldown timers
        let mut slotbar_index = 0;
        for slot_bars in self.slots_usage_last_time {
            let mut slot_index = 0;
            for last_time in slot_bars {
                let cooldown = config
                    .get_slot_cooldown(slotbar_index, slot_index)
                    .unwrap()
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

        // Update avoid bounds cooldowns timers
        let mut result: Vec<(Bounds, Instant, u128)> = vec![];
        for n in 0..self.avoided_bounds.len() {
            let current = self.avoided_bounds[n];
            if current.1.elapsed().as_millis() < current.2 {
                result.push(current);
            }
        }
        self.avoided_bounds = result;

        // Check whether something should be restored
        self.check_restorations(config, image);

        // Use buffs Yiha
        self.check_buffs(config);

        // Check state machine
        self.state = match self.state {
            State::NoEnemyFound => self.on_no_enemy_found(config),
            State::SearchingForEnemy => self.on_searching_for_enemy(config, image),
            State::EnemyFound(mob) => self.on_enemy_found(config, mob, image),
            State::Attacking(mob) => self.on_attacking(config, mob, image),
            State::AfterEnemyKill(_) => self.after_enemy_kill(config),
        }
    }
}

impl<'a> FarmingBehavior<'_> {
    /// Pickup items on the ground.
    fn pickup_items(&mut self, config: &FarmingConfig) {
        use crate::movement::prelude::*;

        let pickup_pet_slot = config.get_slot_index(SlotType::PickupPet);
        let pickup_motion_slot = config.get_slot_index(SlotType::PickupMotion);

        match (pickup_pet_slot, pickup_motion_slot) {
            // Pickup using pet
            (Some(index), _) => {
                if self.last_summon_pet_time.is_none() {
                    //self.slots[index].cooldown = 0;
                    send_slot(index.0, index.1.into());
                    self.last_summon_pet_time = Some(Instant::now());
                }
            }
            // Pickup using motion
            (_, Some(index)) => {
                play!(self.movement => [
                    Repeat(7, vec![
                        // Press the motion key
                        SendSlot(index.0,index.1.into()),
                    ]),
                ]);
            }
            _ => {
                // Do nothing, we have no way to pickup items
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
        if let Some(slot_index) = config.get_usable_slot_index(
            slot_type,
            &mut self.rng,
            threshold,
            self.slots_usage_last_time,
        ) {
            if send {
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

    fn check_restorations(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer) {
        // Check HP
        let stat = Some(image.client_stats.hp.value);
        if self
            .get_slot_for(config, stat, SlotType::Pill, true)
            .is_none()
        {
            self.get_slot_for(config, stat, SlotType::Food, true);
        }

        // Check MP
        let stat = Some(image.client_stats.mp.value);
        self.get_slot_for(config, stat, SlotType::MpRestorer, true);

        // Check FP
        let stat = Some(image.client_stats.fp.value);
        self.get_slot_for(config, stat, SlotType::FpRestorer, true);
    }

    fn check_buffs(&mut self, config: &FarmingConfig) {
        self.get_slot_for(config, None, SlotType::BuffSkill, true);
    }

    fn on_no_enemy_found(&mut self, config: &FarmingConfig) -> State {
        use crate::movement::prelude::*;

        // Try rotating first in order to locate nearby enemies
        if self.rotation_movement_tries < 20 {
            play!(self.movement => [
                // Rotate in random direction for a random duration
                Rotate(rot::Right, dur::Fixed(100)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Fixed(50)),
            ]);
            self.rotation_movement_tries += 1;

            // Transition to next state
            return State::SearchingForEnemy;
        }

        // Check whether bot should stay in area
        if config.should_stay_in_area() {
            // Reset rotation movement tries to keep rotating
            self.rotation_movement_tries = 0;

            // Stay in state
            return self.state;
        }

        // Move in a circle pattern
        send_keystroke(Key::W, KeyMode::Hold);
        send_keystroke(Key::Space, KeyMode::Hold);
        send_keystroke(Key::D, KeyMode::Hold);
        std::thread::sleep(Duration::from_millis(100));
        send_keystroke(Key::D, KeyMode::Release);
        std::thread::sleep(Duration::from_millis(100));
        send_keystroke(Key::Space, KeyMode::Release);
        send_keystroke(Key::W, KeyMode::Release);
        std::thread::sleep(Duration::from_millis(50));

        // Transition to next state
        State::SearchingForEnemy
    }

    fn on_searching_for_enemy(
        &mut self,
        config: &FarmingConfig,
        image: &mut ImageAnalyzer,
    ) -> State {
        if config.is_stop_fighting() {
            return State::Attacking(Target::default());
        }
        let mobs = image.identify_mobs(config);
        if mobs.is_empty() {
            // Transition to next state
            State::NoEnemyFound
        } else {
            // Calculate max distance of mobs
            let max_distance = match config.should_stay_in_area() {
                true => 325,
                false => 1000,
            };

            // Get aggressive mobs to prioritize them
            let mut mob_list = mobs
                .iter()
                .filter(|m| m.target_type == TargetType::Mob(MobType::Aggressive))
                .cloned()
                .collect::<Vec<_>>();
            let mut mob_type = "aggressive";

            // Check if there's aggressive mobs otherwise collect passive mobs
            if mob_list.is_empty() {
                mob_list = mobs
                    .iter()
                    .filter(|m| m.target_type == TargetType::Mob(MobType::Passive))
                    .cloned()
                    .collect::<Vec<_>>();
                mob_type = "passive";
            }

            // Check again
            if !mob_list.is_empty() {
                slog::debug!(self.logger, "Found mobs"; "mob_type" => mob_type, "mob_count" => mob_list.len());
                if let Some(mob) = {
                    // Try avoiding detection of last killed mob
                    if self.avoided_bounds.len() > 0 {
                        slog::debug!(self.logger, "Avoiding mob");
                        image.find_closest_mob(
                            mob_list.as_slice(),
                            Some(&self.avoided_bounds),
                            max_distance,
                            self.logger,
                        )
                    } else {
                        image.find_closest_mob(mob_list.as_slice(), None, max_distance, self.logger)
                    }
                } {
                    // Transition to next state
                    State::EnemyFound(*mob)
                } else {
                    // Transition to next state
                    State::NoEnemyFound
                }
            } else {
                slog::warn!(self.logger, "Mob detection anomaly"; "description" => "mob list is not empty but contains neither aggro nor passive mobs");

                // Transition to next state
                State::NoEnemyFound
            }
        }
    }

    fn on_enemy_found(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &mut ImageAnalyzer,
    ) -> State {
        self.rotation_movement_tries = 0;

        // Transform attack coords into local window coords
        let point = mob.get_attack_coords();

        let target_cursor_pos = Position::Physical(PhysicalPosition {
            x: point.x as i32,
            y: point.y as i32,
        });

        // Set cursor position and simulate a click
        drop(self.platform.window.set_cursor_position(target_cursor_pos));
        std::thread::sleep(Duration::from_millis(100));
        image.capture_window_area(self.logger, config, Area::new(0, 0, 2, 2));
        let cursor_style = PixelDetection::new(PixelDetectionKind::CursorType, Some(image));
        if cursor_style.value {
            drop(
                self.platform
                    .mouse
                    .click(&mouse_rs::types::keys::Keys::LEFT),
            );
            slog::debug!(self.logger, "Trying to attack mob"; "mob_coords" => &point);
            self.missclick_count = 0;
            // Wait a few ms before transitioning state
            std::thread::sleep(Duration::from_millis(100));
            State::Attacking(mob)
        } else {
            self.missclick_count += 1;
            //std::thread::sleep(Duration::from_millis(10));
            self.avoided_bounds
                .push((mob.bounds.grow_by(20), Instant::now(), 500));
            if self.missclick_count == 15 {
                self.missclick_count = 0;
                State::NoEnemyFound
            } else {
                State::SearchingForEnemy
            }
        }
    }

    fn abort_attack(&mut self) -> State {
        use crate::movement::prelude::*;
        play!(self.movement => [
            PressKey(Key::Escape),
        ]);
        return State::SearchingForEnemy;
    }

    fn on_attacking(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &mut ImageAnalyzer,
    ) -> State {
        // Engagin combat
        if (!self.is_attacking && image.client_stats.enemy_hp.value > 0)
            || (!config.is_stop_fighting()
                && PixelDetection::new(PixelDetectionKind::IsNpc, Some(image)).value)
        {
            self.obstacle_avoidance_count = 0;

            // try to implement something related to party, if mob is less than 100% he was probably attacked by someone else so we can avoid it
            if !config.is_stop_fighting()
                && ((config.get_prevent_already_attacked()
                    && image.client_stats.enemy_hp.value < 100)
                    || PixelDetection::new(PixelDetectionKind::IsNpc, Some(image)).value)
            {
                return self.abort_attack();
            }

            self.last_initial_attack_time = Instant::now();
        } else if !self.is_attacking
            && image.client_stats.enemy_hp.value == 0
            && !config.is_stop_fighting()
        {
            use crate::movement::prelude::*;
            play!(self.movement => [
                PressKey(Key::S),
            ]);
        }

        if image.client_stats.enemy_hp.value > 0 {
            self.is_attacking = true;

            // Try to use attack skill if at least one is selected in slot bar
            if let Some(index) = self.get_slot_for(config, None, SlotType::AttackSkill, false) {
                // Helps avoid obstacles only works using attack slot basically try to move after 5sec
                if !config.is_stop_fighting()
                    && image.client_stats.enemy_hp.last_update_time.is_some()
                    && image
                        .client_stats
                        .enemy_hp
                        .last_update_time
                        .unwrap()
                        .elapsed()
                        .as_millis()
                        > 5000
                {
                    // Reset timer otherwise it'll trigger one time
                    image.client_stats.enemy_hp.reset_last_time();

                    // Abort attack after 5 avoidance
                    if self.obstacle_avoidance_count == 5 {
                        return self.abort_attack();
                    }
                    self.last_initial_attack_time = Instant::now();
                    use crate::movement::prelude::*;
                    let rotation_key = [Key::A, Key::D].choose(&mut self.rng).unwrap_or(&Key::A);
                    let rotation_duration = self.rng.gen_range(70_u64..100_u64);
                    let movement_slices = self.rng.gen_range(2..4);

                    // Move into a random direction while jumping
                    play!(self.movement => [
                        HoldKeys(vec![Key::W, Key::Space]),
                        Repeat(movement_slices as u64, vec![
                            HoldKeyFor(*rotation_key, dur::Fixed(rotation_duration)),
                        ]),
                        HoldKeyFor(*rotation_key, dur::Fixed(rotation_duration)),
                        ReleaseKeys(vec![Key::Space, Key::W]),
                    ]);
                    self.obstacle_avoidance_count += 1;
                }

                self.send_slot(index);
            }

            //  Keep attacking until the target marker is gone
            self.state
        } else {
            // Target HP = 0
            if self.is_attacking {
                self.is_attacking = false;
                State::AfterEnemyKill(mob)
            } else {
                State::SearchingForEnemy
            }
        }
    }

    fn after_enemy_kill(&mut self, config: &FarmingConfig) -> State {
        self.kill_count += 1;
        self.last_kill_time = Instant::now();

        // Pickup items
        self.pickup_items(config);

        // Transition state
        State::SearchingForEnemy
    }
}
