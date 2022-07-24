use std::time::{Duration, Instant};

use guard::guard;
use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{Bounds, MobType, Target, TargetType},
    image_analyzer::{ImageAnalyzer, Stat, StatusBar},
    ipc::{BotConfig, FarmingConfig, SlotType},
    movement::MovementAccessor,
    platform::{send_keystroke, Key, KeyMode, PlatformAccessor},
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
    movement: &'a MovementAccessor<'a>,
    state: State,
    last_hp: Stat,
    last_fp: Stat,
    last_mp: Stat,
    last_xp: Stat,
    last_food_hp: Stat,
    last_pot_time: Instant,
    last_initial_attack_time: Instant,
    last_attack_skill_usage_time: Instant,
    last_kill_time: Instant,
    last_killed_mob_bounds: Bounds,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
}

impl<'a> Behavior<'a> for FarmingBehavior<'a> {
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement: &'a MovementAccessor<'a>,
    ) -> Self {
        Self {
            logger,
            platform,
            movement,
            rng: rand::thread_rng(),
            state: State::SearchingForEnemy,
            last_hp: Stat::default(),
            last_fp: Stat::default(),
            last_mp: Stat::default(),
            last_xp: Stat::default(),
            last_food_hp: Stat::default(),
            last_pot_time: Instant::now(),
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            last_killed_mob_bounds: Bounds::default(),
            last_attack_skill_usage_time: Instant::now(),
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(&mut self, config: &BotConfig, image: &ImageAnalyzer) {
        let config = config.farming_config();

        //DEBUG PURPOSE
        self.debug_stats_bar(config, image);
        // Check whether food should be consumed
        self.check_food(config, image);

        // Check state machine
        self.state = match self.state {
            State::Idle => self.on_idle(config),
            State::NoEnemyFound => self.on_no_enemy_found(config),
            State::SearchingForEnemy => self.on_searching_for_enemy(config, image),
            State::EnemyFound(mob) => self.on_enemy_found(config, mob),
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

        match (
            config.should_use_on_demand_pet(),
            pickup_pet_slot,
            pickup_motion_slot,
        ) {
            // Pickup using pet
            (true, Some(index), _) => {
                play!(self.movement => [
                    // Summon pet
                    PressKey(index.into()),
                    // Wait a bit to make sure everything is picked up
                    Wait(dur::Fixed(2000)),
                    // Unsummon pet
                    PressKey(index.into()),
                ]);
            }
            // Pickup using motion
            (false, _, Some(index)) => {
                play!(self.movement => [
                    Repeat(5, vec![
                        // Press the motion key
                        PressKey(index.into()),
                        // Wait a bit
                        Wait(dur::Random(350..750)),
                    ]),
                ]);
            }
            _ => {
                // Do nothing, we have no way to pickup items
            }
        }
    }
    fn debug_stats_bar(&mut self, config: &FarmingConfig, image: &ImageAnalyzer) {
        // Getting all stats
        self.last_fp = image.detect_stats_bar(self.last_fp, StatusBar::Fp).unwrap();

        self.last_mp = image.detect_stats_bar(self.last_mp, StatusBar::Mp).unwrap();

        self.last_hp = image.detect_stats_bar(self.last_hp, StatusBar::Hp).unwrap();

        self.last_xp = image.detect_stats_bar(self.last_xp, StatusBar::Xp).unwrap();

        // Print them
        slog::debug!(self.logger,  "Getting stats ",   ; " " => "");
        slog::debug!(self.logger,  "Trying to detect FP ",   ; "FP PERCENT " =>  self.last_fp.value);
        slog::debug!(self.logger,  "Trying to detect MP ",   ; "MP PERCENT " => self.last_mp.value);
        slog::debug!(self.logger,  "Trying to detect HP ",   ; "HP PERCENT " => self.last_hp.value);
        slog::debug!(self.logger,  "Trying to detect EXP ",   ; "EXP PERCENT " => self.last_xp.value);
    }
    /// Consume food based on HP. Fallback for when HP is unable to be detected.
    fn check_food(&mut self, config: &FarmingConfig, image: &ImageAnalyzer) {
        let current_time = Instant::now();
        let min_pot_time_diff = Duration::from_millis(1000);

        // Decide which fooding logic to use based on HP
        match image.detect_stats_bar(self.last_hp, StatusBar::Hp) {
            // HP bar detected, use HP-based potting logic
            Some(hp) => {
                // HP threshold. We probably shouldn't use food at > 75% HP.
                // If HP is < 15% we need to use food ASAP.
                let hp_threshold_reached = hp.value <= 75;
                let hp_critical_threshold_reached = hp.value <= 15;

                // Calculate ms since last food usage
                let ms_since_last_food = Instant::now()
                    .duration_since(self.last_pot_time)
                    .as_millis();

                // Check whether we can use food again.
                // This is based on a very generous limit of 1s between food uses.
                let can_use_food =
                    current_time.duration_since(self.last_pot_time) > min_pot_time_diff;

                // Use food ASAP if HP is critical.
                // Wait a minimum of 333ms after last usage anyway to avoid detection.
                // Spamming 3 times per second when low on HP seems legit for a real player.
                let should_use_food_reason_hp_critical =
                    ms_since_last_food > 333 && hp_critical_threshold_reached;

                // Use food if nominal usage conditions are met
                let should_use_food_reason_nominal = hp_threshold_reached && can_use_food;

                // Check whether we should use food for any reason
                let should_use_food =
                    should_use_food_reason_hp_critical || should_use_food_reason_nominal;

                // Check whether we should use pills
                let pill_available = config.get_slot_index(SlotType::Pill).is_some();
                let should_use_pill = pill_available && should_use_food_reason_hp_critical;

                if should_use_food {
                    // Use pill
                    if should_use_pill {
                        guard!(let Some(pill_index) = config.get_slot_index(SlotType::Pill) else {
                            self.last_hp = hp;
                            return;
                        });

                        // Send keystroke for first slot mapped to pill
                        send_keystroke(pill_index.into(), KeyMode::Press);

                        // Update state
                        self.last_pot_time = current_time;
                        self.last_food_hp = hp;

                        // wait a few ms for the pill to be consumed
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    // Use regular food
                    else if let Some(food_index) = config.get_slot_index(SlotType::Food) {
                        // Send keystroke for first slot mapped to food
                        send_keystroke(food_index.into(), KeyMode::Press);

                        // Update state
                        self.last_pot_time = current_time;
                        self.last_food_hp = hp;

                        // wait a few ms for the food to be consumed
                        std::thread::sleep(Duration::from_millis(100));
                    } else {
                        slog::info!(self.logger, "No slot is mapped to food!");
                    }
                }

                self.last_hp = hp;
            }

            // HP bar not found, use legacy potting logic
            // => Pot every 5 seconds and only while in a fight
            None => {
                if self.is_attacking
                    && current_time.duration_since(self.last_initial_attack_time)
                        > min_pot_time_diff
                    && current_time.duration_since(self.last_pot_time) > min_pot_time_diff
                {
                    if let Some(food_index) = config.get_slot_index(SlotType::Food) {
                        send_keystroke(food_index.into(), KeyMode::Press);
                        self.last_pot_time = current_time;
                    }
                }
            }
        }
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
                Rotate(rot::Random, dur::Random(100..250)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Random(100..250)),
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
                let rotation_duration = self.rng.gen_range(100_u64..350_u64);
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
                    Wait(dur::Random(1000..4000)),
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

    fn on_searching_for_enemy(&mut self, config: &FarmingConfig, image: &ImageAnalyzer) -> State {
        let mobs = image.identify_mobs();
        if mobs.is_empty() {
            // Transition to next state
            State::NoEnemyFound
        } else {
            // Check if aggro mobs are found first
            let aggro_mobs = mobs
                .iter()
                .filter(|m| m.target_type == TargetType::Mob(MobType::Aggressive))
                .cloned()
                .collect::<Vec<_>>();
            let passive_mobs = mobs
                .iter()
                .filter(|m| m.target_type == TargetType::Mob(MobType::Passive))
                .cloned()
                .collect::<Vec<_>>();

            // Calculate max distance of mobs
            let max_distance = match (config.should_stay_in_area(), config.is_unsupervised()) {
                (_, true) => 300,
                (true, _) => 325,
                (false, false) => 1000,
            };

            // Prioritize aggro mobs
            if !aggro_mobs.is_empty() {
                slog::debug!(self.logger, "Found mobs"; "mob_type" => "aggressive", "mob_count" => aggro_mobs.len());
                let closest_mob = image.find_closest_mob(aggro_mobs.as_slice(), None, max_distance);
                if let Some(mob) = closest_mob {
                    State::EnemyFound(*mob)
                } else {
                    State::NoEnemyFound
                }
            } else if !passive_mobs.is_empty() {
                slog::debug!(self.logger, "Found mobs"; "mob_type" => "passive", "mob_count" => passive_mobs.len());
                if let Some(mob) = {
                    // Try avoiding detection of last killed mob
                    if Instant::now().duration_since(self.last_kill_time)
                        < Duration::from_millis(2500)
                    {
                        slog::debug!(self.logger, "Avoiding mob"; "mob_bounds" => self.last_killed_mob_bounds);
                        image.find_closest_mob(
                            passive_mobs.as_slice(),
                            Some(&self.last_killed_mob_bounds),
                            max_distance,
                        )
                    } else {
                        image.find_closest_mob(passive_mobs.as_slice(), None, max_distance)
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

    fn on_enemy_found(&mut self, _config: &FarmingConfig, mob: Target) -> State {
        self.rotation_movement_tries = 0;

        // Transform attack coords into local window coords
        let point = mob.get_attack_coords();
        slog::debug!(self.logger, "Trying to attack mob"; "mob_coords" => &point);
        // let inner_size = window.inner_size().unwrap();
        // let (x_diff, y_diff) = (
        //     image.width() - inner_size.width,
        //     image.height() - inner_size.height,
        // );
        // let (window_x, window_y) = (
        //     (x.saturating_sub(x_diff / 2)) as i32,
        //     (y.saturating_sub(y_diff)) as i32,
        // );
        let target_cursor_pos = Position::Physical(PhysicalPosition {
            x: point.x as i32,
            y: point.y as i32,
        });

        // Set cursor position and simulate a click
        drop(self.platform.window.set_cursor_position(target_cursor_pos));
        drop(
            self.platform
                .mouse
                .click(&mouse_rs::types::keys::Keys::LEFT),
        );

        // Wait a few ms before transitioning state
        State::Attacking(mob)
    }

    fn on_attacking(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &ImageAnalyzer,
    ) -> State {
        if !self.is_attacking {
            self.last_initial_attack_time = Instant::now();
            self.last_pot_time = Instant::now();
        }
        if let Some(marker) = image.identify_target_marker() {
            // Target marker found
            self.is_attacking = true;
            self.last_killed_mob_bounds = marker.bounds;

            // Try to use attack skill
            if let Some(index) = config.get_random_slot_index(SlotType::AttackSkill, &mut self.rng)
            {
                // Only use attack skill if enabled and once a second at most
                if config.should_use_attack_skills()
                    && self.last_attack_skill_usage_time.elapsed() > Duration::from_secs(1)
                {
                    self.last_attack_skill_usage_time = Instant::now();
                    send_keystroke(index.into(), KeyMode::Press);
                }
            }

            //  Keep attacking until the target marker is gone
            self.state
        } else {
            // Target marker not found
            if self.is_attacking {
                // Enemy was probably killed
                self.is_attacking = false;
                State::AfterEnemyKill(mob)
            } else {
                use crate::movement::prelude::*;
                // Lost target without attacking?
                play!(self.movement => [
                    Wait(dur::Random(1000..2000)),
                ]);
                State::SearchingForEnemy
            }
        }
    }

    fn after_enemy_kill(&mut self, config: &FarmingConfig) -> State {
        self.kill_count += 1;
        self.last_kill_time = Instant::now();

        // Pickup items
        self.pickup_items(config);

        // Check if we're running in unsupervised mode
        if config.is_unsupervised() {
            // Sleep until the killed mob has fully disappeared
            let sleep_time = match config.should_use_on_demand_pet() {
                true => Duration::from_millis(3000),
                false => Duration::from_millis(5000),
            };
            std::thread::sleep(sleep_time);
        }

        // Transition state
        State::SearchingForEnemy
    }
}
