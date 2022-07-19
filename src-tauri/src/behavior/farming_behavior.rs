use std::time::{Duration, Instant};

use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{Bounds, MobType, Target, TargetType},
    image_analyzer::{Hp, ImageAnalyzer},
    ipc::{BotConfig, FarmingConfig, SlotType},
    platform::{send_keystroke, Key, KeyMode, PlatformAccessor},
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
    state: State,
    last_hp: Hp,
    last_food_hp: Hp,
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
    fn new(platform: &'a PlatformAccessor<'a>, logger: &'a Logger) -> Self {
        Self {
            logger,
            platform,
            rng: rand::thread_rng(),
            state: State::SearchingForEnemy,
            last_hp: Hp::default(),
            last_food_hp: Hp::default(),
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

    fn run_iteration(&mut self, config: &BotConfig, img: Option<ImageAnalyzer>) {
        let config = config.farming_config();
        let image: ImageAnalyzer = img.unwrap();
        // Check whether food should be consumed
        self.check_food(config, &image);

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
    /// Consume food based on HP. Fallback for when HP is unable to be detected.
    fn check_food(&mut self, config: &FarmingConfig, image: &ImageAnalyzer) {
        let current_time = Instant::now();
        let min_pot_time_diff = Duration::from_millis(2000);

        // Decide which fooding logic to use based on HP
        match image.detect_hp(self.last_hp) {
            // HP bar detected, use HP-based potting logic
            Some(hp) => {
                // HP threshold. We probably shouldn't use food at > 60% HP.
                // If HP is < 30% we need to use food ASAP.
                let hp_threshold_reached = hp.hp <= 60;
                let hp_critical_threshold_reached = hp.hp <= 30;

                // Calculate ms since last food usage
                let ms_since_last_food = Instant::now()
                    .duration_since(self.last_pot_time)
                    .as_millis();

                // Check whether we can use food again.
                // This is based on a very generous limit of 2s between food uses.
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

                if should_use_food {
                    if let Some(food_index) = config.get_slot_index(SlotType::Food) {
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
        for _ in 0..idle_chunks {
            if self.rng.gen_bool(0.1) {
                send_keystroke(Key::Space, KeyMode::Press);
            }
            std::thread::sleep(idle_chunk_duration);
        }

        // Transition to next state
        State::SearchingForEnemy
    }

    fn on_no_enemy_found(&mut self, config: &FarmingConfig) -> State {
        // Try rotating first in order to locate nearby enemies
        if self.rotation_movement_tries < 20 {
            // Rotate in random direction for a random duration
            let key = [Key::A, Key::D].choose(&mut self.rng).unwrap_or(&Key::A);
            let rotation_duration = std::time::Duration::from_millis(self.rng.gen_range(100..250));
            send_keystroke(*key, KeyMode::Hold);
            std::thread::sleep(rotation_duration);
            send_keystroke(*key, KeyMode::Release);
            self.rotation_movement_tries += 1;
            // Wait a bit to wait for monsters to enter view
            std::thread::sleep(std::time::Duration::from_millis(
                self.rng.gen_range(100..250),
            ));

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
                // Move into a random direction while jumping
                let key = [Key::A, Key::D].choose(&mut self.rng).unwrap_or(&Key::A);
                let rotation_duration = Duration::from_millis(self.rng.gen_range(100..350));
                let movement_slices = self.rng.gen_range(1..4);
                let movement_slice_duration = Duration::from_millis(self.rng.gen_range(250..500));
                let movement_overlap_duration =
                    movement_slice_duration.saturating_sub(rotation_duration);
                send_keystroke(Key::W, KeyMode::Hold);
                send_keystroke(Key::Space, KeyMode::Hold);
                for _ in 0..movement_slices {
                    send_keystroke(*key, KeyMode::Hold);

                    std::thread::sleep(rotation_duration);
                    send_keystroke(*key, KeyMode::Release);
                    std::thread::sleep(movement_overlap_duration);
                }
                send_keystroke(*key, KeyMode::Hold);
                std::thread::sleep(rotation_duration);
                send_keystroke(*key, KeyMode::Release);
                send_keystroke(Key::Space, KeyMode::Release);
                send_keystroke(Key::W, KeyMode::Release);
            }
            1 => {
                // Move forwards while jumping
                send_keystroke(Key::W, KeyMode::Hold);
                send_keystroke(Key::Space, KeyMode::Hold);
                std::thread::sleep(std::time::Duration::from_millis(
                    self.rng.gen_range(1000..4000),
                ));
                send_keystroke(Key::Space, KeyMode::Release);
                send_keystroke(Key::W, KeyMode::Release);
            }
            2 => {
                // Move forwards in a slalom pattern
                let slalom_switch_duration =
                    std::time::Duration::from_millis(self.rng.gen_range(350..650));
                let total_slaloms = self.rng.gen_range(4..8);
                let mut left = self.rng.gen_bool(0.5);
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

    fn on_searching_for_enemy(&mut self, config: &FarmingConfig, image: ImageAnalyzer) -> State {
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
            let max_distance = if config.should_stay_in_area() {
                325
            } else {
                1000
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

    fn on_attacking(&mut self, config: &FarmingConfig, mob: Target, image: ImageAnalyzer) -> State {
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
                // Lost target without attacking?
                State::SearchingForEnemy
            }
        }
    }

    fn after_enemy_kill(&mut self, config: &FarmingConfig) -> State {
        self.kill_count += 1;
        self.last_kill_time = Instant::now();

        // Check for on-demand pet config
        if config.should_use_on_demand_pet() {
            if let Some(index) = config.get_slot_index(SlotType::PickupPet) {
                // Summon pet
                send_keystroke(index.into(), KeyMode::Press);
                // Wait half a second to make sure everything is picked up
                std::thread::sleep(std::time::Duration::from_millis(2000));
                // Unsummon pet
                send_keystroke(index.into(), KeyMode::Press);
            }
        }

        if self.rng.gen_bool(0.1) {
            State::Idle
        } else {
            State::SearchingForEnemy
        }
    }
}
