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
        config.passive_mobs_colors = [234, 234, 149]
    }

    if (!config.passive_tolerence){
        config.passive_tolerence = 2
    }

    if (!config.aggressive_mobs_colors){
        config.aggressive_mobs_colors = [179, 23, 23]
    }

    if (!config.aggressive_tolerence){
        config.aggressive_tolerence = 9
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
                        label={<ConfigLabel name="Colors" helpText="Custom monster name color reference. Edit these values if you are sure what you are doing." />}
                        item={<ColorSelector value={(selectedMobType === 0)? config.passive_mobs_colors : config.aggressive_mobs_colors} onChange={value => onChange?.((selectedMobType === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Tolerence" helpText="Custom monster name color tolerence. Edit these values if you are sure what you are doing." />}
                        item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType === 0)? config.passive_tolerence : config.aggressive_tolerence} onChange={value => onChange?.((selectedMobType === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                    />
                </>

            }/>
            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Avoid already attacked monster (experimental)" helpText="Check whether a mob is already attacked and avoid it if so. Must be disabled if you play in party" />}
                        item={<BooleanSlider value={config.prevent_already_attacked ?? false} onChange={value => onChange?.({ ...config, prevent_already_attacked: value })} />}
                    />
                    <ConfigTableRow
                        disabled={config.unsupervised === true}
                        label={<ConfigLabel name="Stay in Area" helpText="The bot will try to wait in the area and not move around too much." />}
                        item={<BooleanSlider value={config.unsupervised === true ? false : config.stay_in_area ?? false} onChange={value => onChange?.({ ...config, stay_in_area: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Unsupervised (experimental)" helpText="The bot will try extra hard not to move too far. Makes farming a bit slower, but also safer. Enabling this will override the 'Stay in Area' setting." />}
                        item={<BooleanSlider value={config.unsupervised ?? false} onChange={value => onChange?.({ ...config, unsupervised: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Stop mob detection" helpText="Stop mob searching but keeps benefit of using the bot like item pickup, buffs, restoration, etc..." />}
                        item={<BooleanSlider value={config.is_stop_fighting ?? false} onChange={value => onChange?.({ ...config, is_stop_fighting: value })} />}
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
