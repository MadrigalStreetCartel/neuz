export type FixedArray<TItem, TLength extends number> = [TItem, ...TItem[]] & { length: TLength }

export type SlotType = "Unused" | "Food" | "PickupPet" | "PickupMotion" | "AttackSkill" | "BuffSkill" | "Flying" | "Pill"

export type SlotModel = {
    slot_type: SlotType,
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
