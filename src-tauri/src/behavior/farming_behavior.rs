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
    movement::{MovementAccessor, MovementDirection},
    platform::{send_keystroke, send_slot, Key, KeyMode, PlatformAccessor},
    play,
};

use super::Behavior;

#[derive(Debug, Clone, Copy)]
enum State {
    Idle,
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
    last_slots_usage: [Option<Instant>; 10],
    last_initial_attack_time: Instant,
    last_kill_time: Instant,
    last_killed_mob_bounds: Bounds,
    last_killed_mobs_bounds: Vec<(Bounds,Instant, u128)>,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
    obstacle_avoidance_count: u32,
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
            last_slots_usage: [None; 10],
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            last_killed_mob_bounds: Bounds::default(),
            last_killed_mobs_bounds: vec![],
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
            obstacle_avoidance_count: 0,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(&mut self, config: &BotConfig, image: &mut ImageAnalyzer) {
        let config = config.farming_config();

        // Update slot timers
        let mut count = 0;
        for last_time in self.last_slots_usage {
            let cooldown = config.get_slot_cooldown(count).unwrap().try_into();
            if last_time.is_some() && cooldown.is_ok() {
                let slot_last_time = last_time.unwrap().elapsed().as_millis();
                if slot_last_time > cooldown.unwrap() {
                    self.last_slots_usage[count] = None;
                }
            }
            count += 1;
        }
        drop(count);

        // Update avoid bounds
        let mut result: Vec<(Bounds,Instant, u128)> = vec![];
        for n in 0..self.last_killed_mobs_bounds.len() {
            let current = self.last_killed_mobs_bounds[n];
            if current.1.elapsed().as_millis() < current.2 {
                result.push(current);
            }
        }
        self.last_killed_mobs_bounds = result;

        // Check whether something should be used
        self.check_restoration(config, image, StatusBarKind::Hp);
        self.check_restoration(config, image, StatusBarKind::Mp);
        self.check_restoration(config, image, StatusBarKind::Fp);

        // Use buffs Yiha
        self.check_buffs(config);

        // Check state machine
        self.state = match self.state {
            State::Idle => self.on_idle(config),
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
                play!(self.movement => [
                    // Summon pet
                    PressKey(index.into()),
                    // Wait a bit to make sure everything is picked up
                    Wait(dur::Fixed(3000)),
                    // Unsummon pet
                    PressKey(index.into()),
                ]);
            }
            // Pickup using motion
            (_, Some(index)) => {
                play!(self.movement => [
                    Repeat(7, vec![
                        // Press the motion key
                        PressKey(index.into()),
                        // Wait a bit
                        Wait(dur::Random(100..180)),
                    ]),
                ]);
            }
            _ => {
                // Do nothing, we have no way to pickup items
            }
        }

        // Check if we're running in unsupervised mode
        if config.is_unsupervised() {
            // Sleep until the killed mob has fully disappeared
            let sleep_time = if pickup_pet_slot.is_some() {
                Duration::from_millis(3000)
            } else if pickup_motion_slot.is_some() {
                Duration::from_millis(5000)
            } else {
                Duration::from_millis(0)
            };

            std::thread::sleep(sleep_time);
        }
    }

    fn subcheck(
        &mut self,
        config: &FarmingConfig,
        threshold: Option<u32>,
        slot_type: SlotType,
    ) -> bool {
        if let Some(slot_index) =
            config.get_usable_slot_index(slot_type, &mut self.rng, threshold, self.last_slots_usage)
        {
            // Send keystroke for first slot mapped to pill
            send_slot(slot_index.into());

            // Update state
            self.last_slots_usage[slot_index] = Some(Instant::now());

            return true;
        }
        return false;
    }

    fn check_restoration(
        &mut self,
        config: &FarmingConfig,
        image: &mut ImageAnalyzer,
        stat_kind: StatusBarKind,
    ) {
        match stat_kind {
            StatusBarKind::Hp => {
                let stat = Some(image.client_stats.hp.value);
                if !self.subcheck(config, stat, SlotType::Pill) {
                    self.subcheck(config, stat, SlotType::Food);
                }
            }
            StatusBarKind::Mp => {
                let stat = Some(image.client_stats.mp.value);
                self.subcheck(config, stat, SlotType::MpRestorer);
            }
            StatusBarKind::Fp => {
                let stat = Some(image.client_stats.fp.value);
                self.subcheck(config, stat, SlotType::FpRestorer);
            }
            StatusBarKind::Xp => {}
            StatusBarKind::EnemyHp => {}
            StatusBarKind::SpellCasting => {}
        }
    }

    fn check_buffs(&mut self, config: &FarmingConfig) {
        self.subcheck(config, None, SlotType::BuffSkill);
    }

    fn on_idle(&mut self, _config: &FarmingConfig) -> State {
        let total_idle_duration = Duration::from_millis(self.rng.gen_range(500..2000));
        let idle_chunks = self.rng.gen_range(1..4);
        let idle_chunk_duration = total_idle_duration / idle_chunks;

        // Do mostly nothing, but jump sometimes
        self.movement.schedule(|movement| {
            for _ in 0..idle_chunks {
                movement.with_probability(0.1, |movement| {
                    movement.jump();
                });
                std::thread::sleep(idle_chunk_duration);
            }
        });

        // Transition to next state
        State::SearchingForEnemy
    }

    fn on_no_enemy_found(&mut self, config: &FarmingConfig) -> State {
        use crate::movement::prelude::*;

        // Check if we are running fully unsupervised
        if config.is_unsupervised() {
            // Rotate in random direction for a random duration
            play!(self.movement => [
                Rotate(rot::Random, dur::Random(100..300)),
                Wait(dur::Random(100..300)),
            ]);
            self.rotation_movement_tries += 1;

            // Transition to next state
            return State::SearchingForEnemy;
        }

        // Try rotating first in order to locate nearby enemies
        if self.rotation_movement_tries < 20 {
            play!(self.movement => [
                // Rotate in random direction for a random duration
                Rotate(rot::Right, dur::Fixed(200)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Fixed(5)),
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

        // If rotating multiple times failed, try other movement patterns
        match self.rng.gen_range(0..3) {
            0 => {
                let rotation_key = [Key::A, Key::D].choose(&mut self.rng).unwrap_or(&Key::A);
                let rotation_duration = self.rng.gen_range(30_u64..100_u64);
                let movement_slices = self.rng.gen_range(1..4);
                let movement_slice_duration = self.rng.gen_range(250_u64..500_u64);
                let movement_overlap_duration =
                    movement_slice_duration.saturating_sub(rotation_duration);

                // Move into a random direction while jumping
                play!(self.movement => [
                    HoldKeys(vec![Key::W, Key::Space]),
                    Repeat(movement_slices as u64, vec![
                        HoldKeyFor(*rotation_key, dur::Fixed(rotation_duration)),
                        Wait(dur::Fixed(movement_overlap_duration)),
                    ]),
                    HoldKeyFor(*rotation_key, dur::Fixed(rotation_duration)),
                    ReleaseKeys(vec![Key::Space, Key::W]),
                ]);
            }
            1 => {
                // Move forwards while jumping
                play!(self.movement => [
                    HoldKeys(vec![Key::W, Key::Space]),
                    Wait(dur::Random(100..1000)),
                    ReleaseKeys(vec![Key::Space, Key::W]),
                ]);
            }
            2 => {
                let slalom_switch_duration = Duration::from_millis(self.rng.gen_range(350..650));
                let total_slaloms = self.rng.gen_range(4..8);
                let mut left = self.rng.gen_bool(0.5);

                // Move forwards in a slalom pattern
                send_keystroke(Key::W, KeyMode::Hold);
                for _ in 0..total_slaloms {
                    let cond = if left { Key::A } else { Key::D };
                    send_keystroke(cond, KeyMode::Hold);
                    std::thread::sleep(slalom_switch_duration);
                    send_keystroke(cond, KeyMode::Release);
                    left = !left;
                }
                send_keystroke(Key::W, KeyMode::Release);
            }
            _ => unreachable!("Impossible"),
        }

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
            let max_distance = match (config.should_stay_in_area(), config.is_unsupervised()) {
                (_, true) => 300,
                (true, _) => 325,
                (false, false) => 1000,
            };
            let mut mob_list = mobs
                .iter()
                .filter(|m| m.target_type == TargetType::Mob(MobType::Aggressive))
                .cloned()
                .collect::<Vec<_>>();
            let mut mob_type = "aggressive";
            if mob_list.is_empty() {
                mob_list = mobs
                    .iter()
                    .filter(|m| m.target_type == TargetType::Mob(MobType::Passive))
                    .cloned()
                    .collect::<Vec<_>>();
                mob_type = "passive";
            }
            if !mob_list.is_empty() {
                slog::debug!(self.logger, "Found mobs"; "mob_type" => mob_type, "mob_count" => mob_list.len());
                if let Some(mob) = {

                    // Try avoiding detection of last killed mob
                    if self.last_killed_mobs_bounds.len() > 0 {
                    //if self.last_kill_time.elapsed().as_millis() < 5000 {
                        slog::debug!(self.logger, "Avoiding mob");
                        image.find_closest_mob(
                            mob_list.as_slice(),
                            Some(&self.last_killed_mobs_bounds),
                            max_distance,
                        )
                    } else {
                        image.find_closest_mob(mob_list.as_slice(), None, max_distance)
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
        image.capture_window_area(self.logger, config, Area::new(0,0,2,2));
        let cursor_style = PixelDetection::new(PixelDetectionKind::CursorType, Some(image));
        if cursor_style.value {
            drop(
                self.platform
                    .mouse
                    .click(&mouse_rs::types::keys::Keys::LEFT),
            );
            slog::debug!(self.logger, "Trying to attack mob"; "mob_coords" => &point);

            // Wait a few ms before transitioning state
            std::thread::sleep(Duration::from_millis(500));
            State::Attacking(mob)
        } else {
            //self.last_killed_mob_bounds = mob.bounds.grow_by(100);
            self.last_killed_mobs_bounds.push((mob.bounds, Instant::now(), 100));
            State::SearchingForEnemy
        }
    }

    fn abort_attack(&mut self) -> State {
        use crate::movement::prelude::*;
        play!(self.movement => [
            HoldKeyFor(Key::Escape, dur::Fixed(200)),
            //Wait(dur::Random(100..300)),
        ]);
        return State::SearchingForEnemy;
    }

    fn on_attacking(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &mut ImageAnalyzer,
    ) -> State {

        // Target marker found
        let marker = image.identify_target_marker();
        if marker.is_some() {
            let marker = marker.unwrap();
            //self.last_killed_mob_bounds = marker.bounds;
            self.last_killed_mobs_bounds.push((marker.bounds.grow_by(50), Instant::now(), 5000));
        }

        // Engagin combat
        if !self.is_attacking {
            self.obstacle_avoidance_count = 0;

            // try to implement something related to party, if mob is less than 100% he was probably attacked by someone else so we can avoid it
            if !config.is_stop_fighting() && ((config.get_prevent_already_attacked() && image.client_stats.enemy_hp.value < 100)
            || PixelDetection::new(PixelDetectionKind::IsNpc, Some(image)).value) {
                return self.abort_attack();
            }

            self.last_initial_attack_time = Instant::now();
        }

        if image.client_stats.enemy_hp.value > 0 {
            self.is_attacking = true;

            // Try to use attack skill if at least one is selected in slot bar
            if let Some(index) = config.get_usable_slot_index(
                SlotType::AttackSkill,
                &mut self.rng,
                None,
                self.last_slots_usage,
            ) {

                // Helps avoid obstacles only works using attack slot basically try to move after 7.5sec
                if image.client_stats.enemy_hp.last_update_time.is_some() && image.client_stats.enemy_hp.last_update_time.unwrap().elapsed().as_millis() > 7500
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
                    let rotation_duration = self.rng.gen_range(50_u64..150_u64);
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
                send_slot(index.into());
                self.last_slots_usage[index] = Some(Instant::now());
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
