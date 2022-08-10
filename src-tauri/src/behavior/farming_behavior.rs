use std::time::{Duration, Instant};

use guard::guard;
use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{
        Bounds, ClientStats, MobType, PixelDetection, PixelDetectionKind, StatInfo, Target,
        TargetType,
    },
    image_analyzer::ImageAnalyzer,
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
    last_food_cooldown: u32,
    last_food_hp: StatInfo,
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
            last_food_cooldown: 0,
            last_food_hp: StatInfo::default(),
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

    fn run_iteration(&mut self, config: &BotConfig, image: &mut ImageAnalyzer) {
        let config = config.farming_config();

        // Check whether food should be consumed
        self.check_food(config, image);

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

    /// Consume food based on HP. Fallback for when HP is unable to be detected.
    fn check_food(&mut self, config: &FarmingConfig, image: &mut ImageAnalyzer) {
        let current_time = Instant::now();

        // Decide which fooding logic to use based on HP
        let hp = image.client_stats.hp;

        // Calculate ms since last food usage
        let ms_since_last_food = current_time.duration_since(self.last_pot_time).as_millis();

        // Check whether we can use food again.
        // This is based on a very generous limit of 1s between food uses.
        let can_use_food = ms_since_last_food > self.last_food_cooldown.into();

        // Check whether we should use pills
        let pill_available = config
            .get_slot_index_by_threshold(SlotType::Pill, hp.value)
            .is_some();
        let should_use_pill = pill_available && can_use_food;

        if can_use_food {
            // Use pill
            if should_use_pill {
                guard!(let Some(pill_index) = config.get_slot_index_by_threshold(SlotType::Pill, hp.value) else {
                    return;
                });

                // Send keystroke for first slot mapped to pill
                send_keystroke(pill_index.into(), KeyMode::Press);

                // Update state
                self.last_pot_time = current_time;
                self.last_food_cooldown = config.get_slot_cooldown(pill_index) + 2000;

            }
            // Use regular food
            else if let Some(food_index) =
                config.get_slot_index_by_threshold(SlotType::Food, hp.value)
            {
                // Send keystroke for first slot mapped to food
                send_keystroke(food_index.into(), KeyMode::Press);

                // Update state
                self.last_pot_time = current_time;
                self.last_food_cooldown = config.get_slot_cooldown(food_index);

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

    fn on_enemy_found(
        &mut self,
        _config: &FarmingConfig,
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
        image.capture_window(self.logger);
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
            State::SearchingForEnemy
        }
    }

    fn on_attacking(
        &mut self,
        config: &FarmingConfig,
        mob: Target,
        image: &ImageAnalyzer,
    ) -> State {
        //self.client_stats.debug_print();
        if !self.is_attacking {
            // try to implement something related to party, if mob is less than 100% he was probably attacked by someone else
            // true not in party
            // false in party
            /*let prevent_already_attacked = true;
            if prevent_already_attacked && self.client_stats.enemy_hp.value < 100 {
                let marker = image.identify_target_marker();
                if marker.is_some() {
                    let marker = marker.unwrap();
                    self.last_killed_mob_bounds = marker.bounds;
                }
                return State::SearchingForEnemy;
            }*/

            self.last_initial_attack_time = Instant::now();
            self.last_pot_time = Instant::now();
        }
        if image.client_stats.enemy_hp.value > 0 {
            self.is_attacking = true;

            // Target marker found
            let marker = image.identify_target_marker();
            if marker.is_some() {
                let marker = marker.unwrap();
                self.last_killed_mob_bounds = marker.bounds;
            }

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
