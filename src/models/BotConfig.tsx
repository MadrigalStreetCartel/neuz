import IconMotionPickup from '../assets/icon_motion_pickup.png'

export type FixedArray<TItem, TLength extends number> = [TItem, ...TItem[]] & { length: TLength }

export type SlotType = "Unused" | "Food" | "PickupPet" | "PickupMotion" | "AttackSkill" | "BuffSkill" | "Flying" | "Pill"

export const SLOT_SIZE_PX = 40;

export const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return '🍔'
        case 'Pill': return '💊'
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
        case 'PickupPet': return 'Pet'
        case 'PickupMotion': return 'Pickup'
        case 'AttackSkill': return 'Attack'
        case 'BuffSkill': return 'Buff'
        case 'Flying': return 'Board'
    }
}
export type SlotModel = {
    slot_type: SlotType,
    slot_cooldown: number,
    slot_threshold: number,
}

export type SlotBarModel = FixedArray<SlotModel, 10>

export type ModeModel = "Farming" | "Support" | "AutoShout"

export type FarmingConfigModel = Partial<{
    on_demand_pet: boolean,
    use_attack_skills: boolean,
    stay_in_area: boolean,
    unsupervised: boolean,
    slots: SlotBarModel,
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
