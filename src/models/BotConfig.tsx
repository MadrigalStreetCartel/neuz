import IconMotionPickup from '../assets/icon_motion_pickup.png'
import IconVitalDrink from '../assets/icon_vitaldrink.png'
import IconRefresher from '../assets/icon_refresher.png'

export type FixedArray<TItem, TLength extends number> = [TItem, ...TItem[]] & { length: TLength }

export const slotTypes = ["Unused", "Food", "Pill", "MpRestorer", "FpRestorer", "PickupPet", "PickupMotion", "AttackSkill", "BuffSkill", "Flying"] as const;
export const thresholdSlotTypes = ["Food", "Pill", "MpRestorer", "FpRestorer"];
export const cooldownSlotTypes = ["Food", "AttackSkill", "BuffSkill", "Pill", "MpRestorer", "FpRestorer", "PickupPet"];
export type SlotType = typeof slotTypes[number];


export const SLOT_SIZE_PX = 40;

export const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return '🍔'
        case 'Pill': return '💊'
        case 'MpRestorer': return IconRefresher
        case 'FpRestorer': return IconVitalDrink
        case 'PickupPet': return '🐶'
        case 'PickupMotion': return IconMotionPickup
        case 'AttackSkill': return '🗡️'
        case 'BuffSkill': return '🪄'
        case 'Flying': return '✈️'
    }
}

export const translateDesc = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return 'Food'
        case 'Pill': return 'Pill'
        case 'MpRestorer': return 'MP'
        case 'FpRestorer': return 'FP'
        case 'PickupPet': return 'Pet'
        case 'PickupMotion': return 'Pickup'
        case 'AttackSkill': return 'Attack'
        case 'BuffSkill': return 'Buff'
        case 'Flying': return 'Board'
    }
}
export type SlotModel = {
    slot_type: SlotType,
    slot_cooldown?: number,
    slot_threshold?: number,
    slot_enabled: boolean,
}
export type SlotBarHolder = {
    slots: SlotBarModel
}

type SlotBarModel = FixedArray<SlotModel, 10>
export type SlotBars = FixedArray<SlotBarHolder, 9>

export type ModeModel = "Farming" | "Support" | "AutoShout"

export type FarmingConfigModel = Partial<{
    on_demand_pet: boolean,
    use_attack_skills: boolean,
    stay_in_area: boolean,
    slot_bars: SlotBars,
    passive_mobs_colors: number[];
    passive_tolerence: number;
    aggressive_mobs_colors: number[];
    aggressive_tolerence: number;
    is_stop_fighting: boolean;
    prevent_already_attacked: boolean;

    obstacle_avoidance_enabled: boolean,
    obstacle_avoidance_cooldown: number,
    obstacle_avoidance_max_try: number,
    obstacle_avoidance_only_passive: boolean,

    min_mobs_name_width: number,
    max_mobs_name_width: number,

    min_hp_attack: number,

}>

export type SupportConfigModel = Partial<{
    slots: SlotBarModel,
}>

export type ShoutConfigModel = Partial<{
    shout_interval: number,
    shout_messages: string[],
}>

export type BotConfigModel = {
    change_id: number,
    is_running: boolean,
    mode?: ModeModel,
    farming_config: FarmingConfigModel,
    support_config: SupportConfigModel,
    shout_config: ShoutConfigModel,
}
