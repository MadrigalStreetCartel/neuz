import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'

import SlotBar from '../SlotBar'
import { FarmingConfigModel, SlotBarModel, SlotModel } from '../../models/BotConfig'
import NumericInput from '../config/NumericInput'
import ColorSelector from '../config/ColorSelector'
import Modal from '../Modal'
import useModal from '../UseModal'
import { useState } from 'react'

type Props = {
    className?: string,
    config: FarmingConfigModel,
    onChange?: (config: FarmingConfigModel) => void,
}

const createSlotBar = () => (
    [...new Array(10)].map(_ => ({ slot_type: 'Unused', slot_cooldown: 1500, slot_threshold: 30 } as SlotModel)) as SlotBarModel
)

const FarmingConfig = ({ className, config, onChange }: Props) => {
    const handleSlotChange = (index:number, slot:SlotModel) => {
        if (!onChange) return
        const newConfig = { ...config, slots: config.slots ?? createSlotBar() }
        newConfig.slots[index] = slot
        onChange(newConfig)
    }

    if (!config.passive_mobs_colors){
        config.passive_mobs_colors = [232, 232, 148]
    }

    if (!config.passive_tolerence){
        config.passive_tolerence = 2
    }

    if (!config.aggressive_mobs_colors){
        config.aggressive_mobs_colors = [232, 28, 28]
    }

    if (!config.aggressive_tolerence){
        config.aggressive_tolerence = 23
    }

    const { isShowing, toggle } = useModal();
    const [selectedMobType, setSelectedMobType] = useState(0)
    return (
        <>
            <SlotBar slots={config.slots ?? createSlotBar()} onChange={handleSlotChange} />
            <Modal isShowing={isShowing} hide={toggle} title={(selectedMobType === 0)? <h4>Passive mob detection settings</h4> : <h4>Aggressive mob detection settings</h4>} body={
                <>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Colors" helpText="This way you can change detections values, this prevents certains mobs to be untargettable. Edit these values if you are sure what you are doing." />}
                        item={<ColorSelector value={(selectedMobType === 0)? config.passive_mobs_colors : config.aggressive_mobs_colors} onChange={value => onChange?.((selectedMobType === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Tolerence" helpText="Edit these values if you are sure what you are doing." />}
                        item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType === 0)? config.passive_tolerence : config.aggressive_tolerence} onChange={value => onChange?.((selectedMobType === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                    />
                </>

            }/>
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
                        label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(0);
                            toggle();
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Agressive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(1);
                            toggle();
                        }}>⚙️</button>}
                    />

                </ConfigTable>
            </ConfigPanel>
        </>
    )
}

export default styled(FarmingConfig)`

`
