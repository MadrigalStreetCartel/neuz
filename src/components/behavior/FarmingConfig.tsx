import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'

import SlotBar from '../SlotBar'
import { FarmingConfigModel, SlotBarModel, SlotModel, SlotType } from '../../models/BotConfig'
import { useState } from 'react'
import ConfigCollapsible from '../config/ConfigCollapsible'

type Props = {
    className?: string,
    config: FarmingConfigModel,
    onChange?: (config: FarmingConfigModel) => void,
}

const createSlotBar = () => (
    [...new Array(10)].map(_ => ({ slot_type: 'Unused', slot_priority:0 } as SlotModel)) as SlotBarModel
)

const FarmingConfig = ({ className, config, onChange }: Props) => {
    const handleSlotChange = (type: SlotModel, index: number) => {
        if (!onChange ) return
        const newConfig = { ...config, slots: config.slots ?? createSlotBar() }
        newConfig.slots[index] = type ?? {slot_type:"Unused",slot_cooldown:0,slot_priority:0}

        onChange(newConfig)

    }

    return (
        <>
            <SlotBar config={config}  slots={config.slots ?? createSlotBar()} onChange={handleSlotChange} />
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
                    <ConfigTableRow
                        label={<ConfigLabel name="Disable mob searching" helpText="The bot will not try to find and attack mobs but will consume items for HP/MP/FP restoration " />}
                        item={<BooleanSlider value={config.stop_fighting ?? false} onChange={value => onChange?.({ ...config, stop_fighting: value })} />}
                    />
                </ConfigTable>
            </ConfigPanel>
        </>
    )
}

export default styled(FarmingConfig)`

`