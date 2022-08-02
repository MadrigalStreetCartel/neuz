use std::time::{Duration, Instant};

use guard::guard;
use rand::{prelude::SliceRandom, Rng};
use slog::Logger;
use tauri::{PhysicalPosition, Position};

use crate::{
    data::{Bounds, MobType, Target, TargetType},
    image_analyzer::{ImageAnalyzer, StatInfo, StatusBarKind},
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
#[derive(Debug, Clone, Copy)]
pub enum StatValue {
    LastX,
    LastXTime,
    LastUseX,
}

pub struct FarmingBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    platform: &'a PlatformAccessor<'a>,
    movement: &'a MovementAccessor<'a>,
    state: State,

    // HP/FP/MP Detection
    current_status_bar: StatusBarKind,
    last_hp: StatInfo,
    last_fp: StatInfo,
    last_mp: StatInfo,
    last_xp: StatInfo,
    last_food_hp: StatInfo,
    last_food_fp: StatInfo,
    last_food_mp: StatInfo,
    last_hp_time: Instant,
    last_fp_time: Instant,
    last_mp_time: Instant,
    last_slots_usage: [Option<Instant>; 10],

    // Attack
    last_initial_attack_time: Instant,
    last_kill_time: Instant,
    last_killed_mob_bounds: Bounds,
    rotation_movement_tries: u32,
    is_attacking: bool,
    kill_count: u32,
    last_enemy_hp:StatInfo,
    enemy_clicked: bool,
    enemy_last_clicked:Instant,
    spell_cast:StatInfo,
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
            last_hp: StatInfo::default(),
            last_fp: StatInfo::default(),
            last_mp: StatInfo::default(),
            last_xp: StatInfo::default(),
            last_food_hp: StatInfo::default(),
            last_food_mp: StatInfo::default(),
            last_food_fp: StatInfo::default(),
            last_hp_time: Instant::now(),
            last_fp_time: Instant::now(),
            last_mp_time: Instant::now(),
            last_slots_usage: [None; 10],

            // Attack
            last_initial_attack_time: Instant::now(),
            last_kill_time: Instant::now(),
            last_killed_mob_bounds: Bounds::default(),
            is_attacking: false,
            rotation_movement_tries: 0,
            kill_count: 0,
            last_enemy_hp: StatInfo::default(),
            enemy_clicked:false,
            enemy_last_clicked:Instant::now(),
            spell_cast:StatInfo::default(),
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(
        &mut self,
        config: &BotConfig,
        image: &ImageAnalyzer,
        hp: StatInfo,
        mp: StatInfo,
        fp: StatInfo,
        enemy_hp: StatInfo,
        spell_cast:StatInfo,
    ) {
        let config = config.farming_config();

        self.last_hp = hp;
        self.last_food_hp = hp;

        self.last_mp = mp;
        self.last_food_mp = mp;

        self.last_fp = fp;
        self.last_food_fp = fp;

        self.last_enemy_hp = enemy_hp;

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

        //DEBUG PURPOSE
        #[cfg(debug_assertions)]
        self.debug_stats_bars();

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
                    Wait(dur::Random(200..300)),
                    Repeat(rnd, vec![
                        // Press the motion key
                        PressKey(index.into()),
                        // Wait a bit
                        Wait(dur::Random(100..200)),
                    ]),
                ]);
            }
            _ => {
                // Do nothing, we have no way to pickup items
            }
        }
    }

    // Debug values
    #[cfg(debug_assertions)]
    fn debug_stats_bar(
        &self,
        last_value: StatInfo,
        bar: StatusBarKind,
        last_food_x: StatInfo,
        last_x_time: Instant,
    ) {
        slog::debug!(self.logger, "Stats";"Currently detecting" => self.stat_name(bar),
            "last_value.value" => last_value.value,
            "last_value.max_w" => last_value.max_w,
            "last_food_x.value" => last_food_x.value,
            "last_food_x.max_w" => last_food_x.max_w,
            "last_x_time" => last_x_time.elapsed().as_secs());
    }

    #[cfg(debug_assertions)]
    fn debug_stats_bars(&self) {
        // Print stats
        if true {
            let enemy_found = self.last_enemy_hp.value > 0;
            let enemy_result = {
                if !enemy_found {
                    "No enemy targeted".to_string()
                }else{
                    self.last_enemy_hp.value.to_string()
                }
            };

            slog::debug!(self.logger,  "Trying to detect stats ",   ;
                "ENEMY HP PERCENT " => enemy_result,
                "FP PERCENT " =>  self.last_fp.value,
                "MP PERCENT " => self.last_mp.value,
                "HP PERCENT " => self.last_hp.value,
                "EXP PERCENT " => self.last_xp.value
            );
        }
    }

    #[cfg(debug_assertions)]
    fn debug_threshold_reached(&self,  config:&FarmingConfig , bar_name:&str , value: u32,slot_index:usize) {
        slog::debug!(self.logger,  "Stats ",   ;
        "Threshold reached for " => bar_name,
        "value" => value,
        "Triggered slot " => slot_index,
        "Slot threshold " => config.get_slot_threshold(slot_index));
    }

    fn stat_name(&self, bar: StatusBarKind) -> String {
        let str = match bar {
            StatusBarKind::Hp => "HP",
            StatusBarKind::Fp => "FP",
            StatusBarKind::Mp => "MP",
            StatusBarKind::Xp => "EXP",
            StatusBarKind::EnemyHp => "Enemy HP",
            StatusBarKind::SpellCasting => "Spell casting"
        };
        return str.to_string();
    }

    fn check_bar(&mut self, config: &FarmingConfig, bar: StatusBarKind) {
        // Check wich bar is asked
        match bar {
            StatusBarKind::Hp => {
                let slot_available = config
                    .get_slot_index(SlotType::Pill, self.last_slots_usage)
                    .is_some();
                let should_use = slot_available;

                if should_use {
                    guard!(let Some(slot_index) = config.get_slot_index(SlotType::Pill,self.last_slots_usage) else {
                        return;
                    });
                    // Send keystroke for first mapped slot
                    self.check_mp_fp_hp(config, StatusBarKind::Hp, slot_index);
                }
                // Use regular food
                else if let Some(slot_index) =
                    config.get_slot_index(SlotType::Food, self.last_slots_usage)
                {
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
                    self.check_mp_fp_hp(config, StatusBarKind::Mp, slot_index);
                } else {
                    //slog::info!(self.logger, "No slot is mapped to FP!");
                }
            }
            StatusBarKind::Xp => {/* Well nothing to do */},
            StatusBarKind::EnemyHp => {/* Well nothing to do */}
            StatusBarKind::SpellCasting => {}
        }
    }

    fn update_stats(&mut self, ctype: StatusBarKind, value: StatInfo, update_time: bool) {
        self.stats_values(ctype, StatValue::LastX, true, value);
        self.stats_values(ctype, StatValue::LastUseX, true, value);

        if update_time {
            self.stats_values(ctype, StatValue::LastXTime, true, value);
        }
    }

    fn stats_values(
        &mut self,
        ctype: StatusBarKind,
        stat_value: StatValue,
        set_val: bool,
        value: StatInfo,
    ) -> (StatInfo, Option<Instant>) {
        let current_time = Instant::now();

        let mut current_value: StatInfo = StatInfo::default();
        let mut current_value_time: Option<Instant> = None;

        match ctype {
            StatusBarKind::Hp => {
                if set_val {
                    match stat_value {
                        StatValue::LastX => self.last_hp = value,
                        StatValue::LastUseX => self.last_food_hp = value,
                        StatValue::LastXTime => self.last_hp_time = current_time,
                    };
                } else {
                    match stat_value {
                        StatValue::LastX => current_value = self.last_hp,
                        StatValue::LastUseX => current_value = self.last_food_hp,
                        StatValue::LastXTime => current_value_time = Some(self.last_hp_time),
                    };
                }
                self.current_status_bar = StatusBarKind::Hp
            }
            StatusBarKind::Mp => {
                if set_val {
                    match stat_value {
                        StatValue::LastX => self.last_mp = value,
                        StatValue::LastUseX => self.last_food_mp = value,
                        StatValue::LastXTime => self.last_mp_time = current_time,
                    };
                } else {
                    match stat_value {
                        StatValue::LastX => current_value = self.last_mp,
                        StatValue::LastUseX => current_value = self.last_food_mp,
                        StatValue::LastXTime => current_value_time = Some(self.last_mp_time),
                    };
                }
                self.current_status_bar = StatusBarKind::Hp
            }
            StatusBarKind::Fp => {
                if set_val {
                    match stat_value {
                        StatValue::LastX => self.last_fp = value,
                        StatValue::LastUseX => self.last_food_fp = value,
                        StatValue::LastXTime => self.last_fp_time = current_time,
                    };
                } else {
                    match stat_value {
                        StatValue::LastX => current_value = self.last_fp,
                        StatValue::LastUseX => current_value = self.last_food_fp,
                        StatValue::LastXTime => current_value_time = Some(self.last_fp_time),
                    };
                }
                self.current_status_bar = StatusBarKind::Hp
            }
            StatusBarKind::Xp => {
                current_value = self.last_xp;
                current_value_time = Some(current_time);
            }
            StatusBarKind::EnemyHp => {
                current_value = self.last_enemy_hp;
                current_value_time = Some(current_time);
            }
            StatusBarKind::SpellCasting => {
                current_value = self.spell_cast;
                current_value_time = Some(current_time);
            }
        };

        (current_value, current_value_time)
    }

    /// Consume based on value.
    fn check_mp_fp_hp(&mut self, config: &FarmingConfig, bar: StatusBarKind, slot_index: usize) {
        let threshold = config.get_slot_threshold(slot_index).unwrap_or(60);

        let mut x_value = self
            .stats_values(bar, StatValue::LastX, false, StatInfo::default())
            .0;
        let last_x = self
            .stats_values(bar, StatValue::LastUseX, false, StatInfo::default())
            .0;
        let last_x_time = self.stats_values(bar, StatValue::LastXTime, false, StatInfo::default());

        // HP threshold. We probably shouldn't use food at > 75% HP.
        // If HP is < 15% we need to use food ASAP.
        let hp_threshold_reached = x_value.value <= threshold;
        let critical_threshold = (100.0 - (threshold as f32 * 0.75)) as u32;
        let hp_critical_threshold_reached = x_value.value <= critical_threshold;

        // Wait a minimum of 333ms after last usage anyway to avoid detection.
        // Spamming 3 times per second when low on HP seems legit for a real player.
        let should_use_food_reason_hp_critical = hp_critical_threshold_reached;

        // Use food if nominal usage conditions are met
        let should_use_food_reason_nominal = hp_threshold_reached;

        // Check whether we should use food for any reason
        let should_use_food = should_use_food_reason_hp_critical || should_use_food_reason_nominal;

        if false {
            #[cfg(debug_assertions)]
            self.debug_stats_bar(last_x, bar, last_x_time.0, last_x_time.1.unwrap());
        }

        // Never trigger something if we're dead
        if should_use_food && last_x.value > 0 {
            #[cfg(debug_assertions)]
            self.debug_threshold_reached(config,self.stat_name(bar).as_str(),x_value.value,slot_index);

            // Send keystroke for first slot mapped
            send_keystroke(slot_index.into(), KeyMode::Press);

            // Update slot last time
            self.last_slots_usage[slot_index] = Some(Instant::now());

            // wait a few ms for the pill to be consumed
            std::thread::sleep(Duration::from_millis(500));
            //x_value = StatInfo { max_w: x_value.max_w, value:100};
        }

        self.update_stats(bar, x_value, false);
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
        if !self.enemy_clicked {
            // Set cursor position and simulate a click
            drop(self.platform.window.set_cursor_position(target_cursor_pos));
            drop(
                self.platform
                    .mouse
                    .click(&mouse_rs::types::keys::Keys::LEFT),
            );
            self.enemy_clicked = true;
            self.enemy_last_clicked = Instant::now();
            self.state
        }else{
            if self.last_enemy_hp.value == 0 {
                if self.enemy_last_clicked.elapsed().as_millis() > 3000{
                    self.enemy_clicked = false;
                    return State::SearchingForEnemy;
                }
                return self.state;
            }
            self.enemy_clicked = false;
            if self.last_enemy_hp.value == 100 || false && self.last_enemy_hp.value > 0 /*If player is same party*/{
                State::Attacking(mob)
            }else {
                use crate::movement::prelude::*;
                play!(self.movement => [
                    Wait(dur::Random(300..500)),
                ]);
                State::SearchingForEnemy
            }
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
            self.last_hp_time = Instant::now();
        }
        if  self.last_enemy_hp.value > 0{
            // If mob not loose any hp try to jump -> problem it stops char running on ennemy
            /*if self.last_initial_attack_time.elapsed().as_millis() > 100 && self.last_enemy_hp.value == 100 {
                use crate::movement::prelude::*;
                play!(self.movement => [
                    PressKey(Key::Space),
                    Wait(dur::Random(100..300)),
                    PressKey(Key::Space),
                ]);
            }*/

            // Target marker found
            self.is_attacking = true;
            let marker = image.identify_target_marker();
            if marker.is_some() {
                let marker = marker.unwrap();
                self.last_killed_mob_bounds = marker.bounds;
            }

            // Only use attack skill if enabled and once a second at most
            if config.should_use_attack_skills() {
                // Try to use attack skill'à
                if let Some(slot_index) = config.get_random_slot_index(
                    SlotType::AttackSkill,
                    &mut self.rng,
                    self.last_slots_usage,
                ) {

                    send_keystroke(slot_index.into(), KeyMode::Press);
                    std::thread::sleep(Duration::from_millis(1000));
                    self.last_slots_usage[slot_index] = Some(Instant::now());
                }
            }

            //  Keep attacking until the target marker is gone
            self.state
        } else {
            // Target marker not found
            if self.is_attacking && self.last_enemy_hp.value == 0 {
                // Enemy was probably killed
                self.is_attacking = false;
                State::AfterEnemyKill(mob)
            } else {
                use crate::movement::prelude::*;
                // Lost target without attacking?
                play!(self.movement => [
                    Wait(dur::Random(300..500)),
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
