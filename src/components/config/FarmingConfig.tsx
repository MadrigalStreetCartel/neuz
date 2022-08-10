import styled from 'styled-components'

import BooleanSlider from './BooleanSlider'
import ConfigLabel from './ConfigLabel'
import ConfigPanel from './ConfigPanel'
import ConfigTable from './ConfigTable'
import ConfigTableRow from './ConfigTableRow'

import SlotBar from '../SlotBar'
import { FarmingConfigModel, SlotBarModel, SlotModel, SlotType } from '../../models/BotConfig'

type Props = {
    className?: string,
    config: FarmingConfigModel,
    onChange?: (config: FarmingConfigModel) => void,
}

const createSlotBar = () => (
    [...new Array(10)].map(_ => ({ slot_type: 'Unused', slot_cooldown: 1500, slot_threshold: 30 } as SlotModel)) as SlotBarModel
)

const FarmingConfig = ({ className, config, onChange }: Props) => {
    const handleSlotChange = (type: SlotType, index: number) => {
        if (!onChange) return
        const newConfig = { ...config, slots: config.slots ?? createSlotBar() }
        newConfig.slots[index] = { slot_type: type, slot_cooldown: 1500, slot_threshold: 30 }
        onChange(newConfig)
    }

    const handleSlotUpdateParams = (index:number, slot:SlotModel) => {
        if (!onChange) return
        const newConfig = { ...config, slots: config.slots ?? createSlotBar() }
        newConfig.slots[index] = slot
        onChange(newConfig)
    }

    return (
        <>
            <SlotBar slots={config.slots ?? createSlotBar()} updateSlot={handleSlotUpdateParams} onChange={handleSlotChange} />
            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="On-Demand Pickup Pet" helpText="Manages pickup pet activation automatically. Make sure the pet is NOT summoned before starting." />}
                        item={<BooleanSlider value={config.on_demand_pet ?? false} onChange={value => onChange?.({ ...config, on_demand_pet: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Use Skills to Attack" helpText="Enables the use of skills configured in the action slot to attack. DO NOT ACTIVATE unless you got a Refresher Hold or Vital Drink X active (depending on whether it's an MP or FP skill)." />}
                        item={<BooleanSlider value={config.use_attack_skills ?? false} onChange={value => onChange?.({ ...config, use_attack_skills: value })} />}
                    />
                    <ConfigTableRow
                        disabled={config.unsupervised === true}
                        label={<ConfigLabel name="Stay in Area" helpText="The bot will try to wait in the area and not move around too much." />}
                        item={<BooleanSlider value={config.unsupervised === true ? false : config.stay_in_area ?? false} onChange={value => onChange?.({ ...config, stay_in_area: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Unsupervised (Experimental)" helpText="The bot will try extra hard not to move too far. Makes farming a bit slower, but also safer. Enabling this will override the 'Stay in Area' setting." />}
                        item={<BooleanSlider value={config.unsupervised ?? false} onChange={value => onChange?.({ ...config, unsupervised: value })} />}
                    />
                </ConfigTable>
            </ConfigPanel>
        </>
    )
}

export default styled(FarmingConfig)`

`
