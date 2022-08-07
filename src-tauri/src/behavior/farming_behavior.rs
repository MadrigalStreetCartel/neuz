use std::time::{Duration, Instant};

use guard::guard;
use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};
use libscreenshot::WindowCaptureProvider;

use crate::{
    data::{Bounds, MobType, StatInfo, StatsDetection, StatusBarKind, Target, TargetType, PixelDetectionKind, PixelDetectionInfo},
    image_analyzer::ImageAnalyzer,
    ipc::{BotConfig, FarmingConfig, SlotType},
    movement::MovementAccessor,
    platform::{send_keystroke, Key, KeyMode, PlatformAccessor},
    play, utils::Timer,
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

    // HP/FP/MP Detection
    current_status_bar: StatusBarKind,
    stats_detection: StatsDetection,
    last_slots_usage: [Option<Instant>; 10],
    is_cursor_attack: PixelDetectionInfo,
    mouse_moved:bool,
    window_id:Option<u64>,


    // Attack
    last_initial_attack_time: Instant,
    last_kill_time: Instant,
    last_killed_mob_bounds: Bounds,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
    enemy_last_clicked: Instant,
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

            // HP/FP/MP Detection
            current_status_bar: StatusBarKind::default(),
            stats_detection: StatsDetection::default(),
            last_slots_usage: [None; 10],
            is_cursor_attack: PixelDetectionInfo::default(),
            mouse_moved:false,
            window_id:None,

            // Attack
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            last_killed_mob_bounds: Bounds::default(),
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
            enemy_last_clicked: Instant::now(),
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(
        &mut self,
        config: &BotConfig,
        image: &ImageAnalyzer,
        stats_detection: &mut StatsDetection,
        is_cursor_attack: &mut PixelDetectionInfo,
        window_id: Option<u64>,
    ) {
        let config = config.farming_config();
        self.stats_detection = stats_detection.clone();
        self.is_cursor_attack = is_cursor_attack.clone();
        self.window_id = window_id;


        // Update slot timers
        let mut count = 0;
        for last_time in self.last_slots_usage {
            if config.get_slot_cooldown(count).is_some() {
                let cooldown = config.get_slot_cooldown(count).unwrap().try_into();
                if last_time.is_some() && cooldown.is_ok() {
                    let slot_last_time = last_time.unwrap().elapsed().as_millis();
                    if slot_last_time > cooldown.unwrap() {
                        self.last_slots_usage[count] = None;
                    }
                }
            }
            count += 1;
        }

        //DEBUG PURPOSE
        #[cfg(debug_assertions)]
        self.debug_stats();

        // Check state machine
        self.state = match self.state {
            State::Idle => self.on_idle(config),
            State::NoEnemyFound => self.on_no_enemy_found(config),
            State::SearchingForEnemy => self.on_searching_for_enemy(config, image),
            State::EnemyFound(mob) => self.on_enemy_found(config, mob),
            State::Attacking(mob) => self.on_attacking(config, mob, image),
            State::AfterEnemyKill(_) => self.after_enemy_kill(config),
        };

        // Check whether something should be consumed
        if config
            .get_slot_index(SlotType::Refresher, self.last_slots_usage)
            .is_some()
        {
            self.check_bar(config, StatusBarKind::Mp);
        }

        if config
            .get_slot_index(SlotType::Pill, self.last_slots_usage)
            .is_some()
            || config
                .get_slot_index(SlotType::Food, self.last_slots_usage)
                .is_some()
        {
            self.check_bar(config, StatusBarKind::Hp);
        }

        if config
            .get_slot_index(SlotType::VitalDrink, self.last_slots_usage)
            .is_some()
        {
            self.check_bar(config, StatusBarKind::Fp);
        }



    }
}

impl<'a> FarmingBehavior<'_> {

    /// Capture the current window contents.
    fn capture_window(&mut self, window_id: Option<u64>) -> Option<ImageAnalyzer> {
        let _timer = Timer::start_new("capture_window");
        if let Some(provider) = libscreenshot::get_window_capture_provider() {
            if let Some(window_id) = window_id {
                if let Ok(image) = provider.capture_window(window_id) {
                    Some(ImageAnalyzer::new(image))
                } else {
                    slog::warn!(self.logger, "Failed to capture window"; "window_id" => window_id);
                    None
                }
            } else {
                slog::warn!(self.logger, "Failed to obtain window id");
                None
            }
        } else {
            None
        }
    }
    /// Pickup items on the ground.
    fn pickup_items(&mut self, config: &FarmingConfig) {
        use crate::movement::prelude::*;

        let pickup_pet_slot = config.get_slot_index(SlotType::PickupPet, self.last_slots_usage);
        let pickup_motion_slot =
            config.get_slot_index(SlotType::PickupMotion, self.last_slots_usage);

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
                let rnd = self.rng.gen_range(7..10);
                play!(self.movement => [
                    Wait(dur::Random(300..400)),
                    Repeat(rnd, vec![
                        // Press the motion key
                        PressKey(index.into()),
                        // Wait a bit
                        Wait(dur::Random(200..300)),
                    ]),
                ]);
            }
            _ => {
                // Do nothing, we have no way to pickup items
            }
        }
    }

    #[cfg(debug_assertions)]
    fn debug_stats(&self) {
        // Print stats
        if false {
            let enemy_found = self.stats_detection.enemy_hp.value > 0;
            let enemy_result = {
                if !enemy_found {
                    "No enemy targeted".to_string()
                } else {
                    self.stats_detection.enemy_hp.value.to_string()
                }
            };

            slog::debug!(self.logger,  "Stats",   ;
                "Cast spelling " => self.stats_detection.spell_cast.value, // Maybe we can use this ?
                "ENEMY HP PERCENT " => enemy_result,
                "EXP PERCENT " => self.stats_detection.xp.value,
                "FP PERCENT " =>  self.stats_detection.hp.value,
                "MP PERCENT " => self.stats_detection.mp.value,
                "HP PERCENT " => self.stats_detection.hp.value,

            );
        }
    }

    fn check_bar(&mut self, config: &FarmingConfig, bar: StatusBarKind) {
        // Check wich bar is asked
        match bar {
            StatusBarKind::Hp => {
                let slot_available_pill = config
                    .get_slot_index(SlotType::Pill, self.last_slots_usage)
                    .is_some();
                let slot_available_food = config
                    .get_slot_index(SlotType::Food, self.last_slots_usage)
                    .is_some();

                let should_use = slot_available_pill || slot_available_food;

                if should_use {
                    let slot_index;
                    if slot_available_pill {
                        guard!(let Some(slot_indexx) = config.get_slot_index(SlotType::Pill,self.last_slots_usage) else {
                            return;
                        });
                        slot_index = slot_indexx;
                    } else if slot_available_food {
                        guard!(let Some(slot_indexx) = config.get_slot_index(SlotType::Food,self.last_slots_usage) else {
                            return;
                        });
                        slot_index = slot_indexx;
                    } else {
                        return;
                    }
                    // Send keystroke for first mapped slot
                    self.check_mp_fp_hp(config, StatusBarKind::Hp, slot_index);
                } else {
                    //slog::info!(self.logger, "No slot is mapped to HP!");
                }
            }
            StatusBarKind::Mp => {
                let slot_available = config
                    .get_slot_index(SlotType::Refresher, self.last_slots_usage)
                    .is_some();
                let should_use = slot_available;

                if should_use {
                    guard!(let Some(slot_index) = config.get_slot_index(SlotType::Refresher,self.last_slots_usage) else {
                        return;
                    });
                    // Send keystroke for first slot mapped
                    self.check_mp_fp_hp(config, StatusBarKind::Mp, slot_index);
                } else {
                    // slog::info!(self.logger, "No slot is mapped to MP!");
                }
            }
            StatusBarKind::Fp => {
                let slot_available = config
                    .get_slot_index(SlotType::VitalDrink, self.last_slots_usage)
                    .is_some();
                let should_use = slot_available;

                if should_use {
                    guard!(let Some(slot_index) = config.get_slot_index(SlotType::VitalDrink,self.last_slots_usage) else {
                        return;
                    });
                    // Send keystroke for first slot mapped
                    self.check_mp_fp_hp(config, StatusBarKind::Fp, slot_index);
                } else {
                    //slog::info!(self.logger, "No slot is mapped to FP!");
                }
            }
            StatusBarKind::Xp => { /* Well nothing to do */ }
            StatusBarKind::EnemyHp => { /* Well nothing to do */ }
            StatusBarKind::SpellCasting => {}
        }
    }

    fn stats_values(&mut self, ctype: StatusBarKind) -> &mut StatInfo {
        match ctype {
            StatusBarKind::Hp => {
                self.current_status_bar = StatusBarKind::Hp;
                return &mut self.stats_detection.hp;
            }
            StatusBarKind::Mp => {
                self.current_status_bar = StatusBarKind::Mp;
                return &mut self.stats_detection.mp;
            }
            StatusBarKind::Fp => {
                self.current_status_bar = StatusBarKind::Fp;
                return &mut self.stats_detection.fp;
            }
            StatusBarKind::Xp => {
                return &mut self.stats_detection.xp;
            }
            StatusBarKind::EnemyHp => {
                return &mut self.stats_detection.enemy_hp;
            }
            StatusBarKind::SpellCasting => {
                return &mut self.stats_detection.spell_cast;
            }
        };
    }

    /// Consume based on value.
    fn check_mp_fp_hp(&mut self, config: &FarmingConfig, bar: StatusBarKind, slot_index: usize) {
        let threshold = config.get_slot_threshold(slot_index).unwrap_or(60);

        let mut last_x = self.stats_values(bar);

        // Check whether we should use food for any reason
        let should_use_food = last_x.value <= threshold  && last_x.value < 100;

        // Never trigger something if we're dead
        if should_use_food && last_x.value > 0 && last_x.last_item_used_time.is_none()  {
            /*#[cfg(debug_assertions)]
            // Debug broken don't want to know why
            if false {
                slog::debug!(self.logger,  "Stats ",   ;
                "Threshold reached for " => bar.to_string(),
                "value" => last_x.value,
                "Triggered slot " => slot_index,
                "Slot threshold " => config.get_slot_threshold(slot_index));
            }*/

            // Send keystroke for first slot mapped
            send_keystroke(slot_index.into(), KeyMode::Press);
            last_x.last_item_used_time = Some(Instant::now());

            // Update slot last time
            self.last_slots_usage[slot_index] = Some(Instant::now());
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
        if config.is_stop_fighting() {
            return State::Attacking(Target::default());
        }
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
                        < Duration::from_millis(3000)
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

        if let Some(image) = self.capture_window(self.window_id) {
            self.is_cursor_attack.update_value(&image);

            if self.is_cursor_attack.value {
                drop(
                    self.platform
                        .mouse
                        .click(&mouse_rs::types::keys::Keys::LEFT),
                );

                slog::debug!(self.logger, "Trying to attack mob"; "mob_coords" => &point);

                self.enemy_last_clicked = Instant::now();

                State::Attacking(mob)
            } else {
                State::SearchingForEnemy
            }
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
        if !self.is_attacking {
            self.last_initial_attack_time = Instant::now();
        }
        let image = self.capture_window(self.window_id).unwrap();
        if let Some(marker) = image.identify_target_marker()  {

            self.is_attacking = true;

            // Target marker
            self.last_killed_mob_bounds = marker.bounds;

            // WIP : Will need to add attack motion with a very low cooldown in order to work, or do another way
            /*if self
                    .stats_detection
                    .enemy_hp
                    .last_update_time
                    .unwrap()
                    .elapsed()
                    .as_millis()
                    > 4000
                {
                    use crate::movement::prelude::*;

                    play!(self.movement => [
                        HoldKeyFor(Key::Space,dur::Random(100..250)),
                    ]);
                }*/

            // Only use attack skill if enabled and once a second at most
            if config.should_use_attack_skills() && self.stats_detection.enemy_hp.value < 100 {
                // Try to use attack skill'à
                if let Some(slot_index) = config.get_random_slot_index(
                    SlotType::AttackSkill,
                    &mut self.rng,
                    self.last_slots_usage,
                ) {
                    send_keystroke(slot_index.into(), KeyMode::Press);
                    self.last_slots_usage[slot_index] = Some(Instant::now());
                }
            }

            //  Keep attacking until the target marker is gone
            self.state
        } else {
            // Target marker not found
            if self.is_attacking {
                // Enemy was probably killed
                self.is_attacking = false;
                self.last_kill_time = Instant::now();
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
