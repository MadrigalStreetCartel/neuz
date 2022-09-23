import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'

import SlotBar from '../SlotBar'
import { FarmingConfigModel, SlotBarHolder, SlotBars, SlotModel } from '../../models/BotConfig'
import NumericInput from '../config/NumericInput'
import ColorSelector from '../config/ColorSelector'
import Modal from '../Modal'
import useModal from '../UseModal'
import { useState } from 'react'

type Props = {
    className?: string,
    config: FarmingConfigModel,
    onChange: (config: FarmingConfigModel) => void,
}

const createSlotBars = () => (
    [...new Array(9)].map(_ => ({slots:[...new Array(10)].map(_ => ({ slot_type: 'Unused', /* slot_cooldown: 1000, slot_threshold: 100 ,*/ slot_enabled: true } as SlotModel))})) as SlotBars
)

const FarmingConfig = ({ className, config, onChange }: Props) => {
    const handleSlotChange = (slot_bar_index:number, slot_index:number, slot: SlotModel) => {
        const newConfig = { ...config, slot_bars: config.slot_bars ?? createSlotBars() }
        newConfig.slot_bars[slot_bar_index].slots[slot_index] = slot
        onChange(newConfig)
    }
    const debugModal = useModal();
    const mobsDebugModal = useModal(debugModal);



    const [selectedMobType, setSelectedMobType] = useState(0)
    const defaultPassiveValues = {passive_mobs_colors: [234, 234, 149], passive_tolerence: 5}
    const defaultAggressiveValues = {aggressive_mobs_colors: [179, 23, 23], aggressive_tolerence: 10}
    const resetColorsRefs = (both?:boolean) => {


        const resetPassive = () => onChange({...config, ...defaultPassiveValues })
        const resetAggressive = () => onChange({...config, ...defaultAggressiveValues })
        const resetBoth = () => onChange({...config, ...defaultAggressiveValues, ...defaultPassiveValues})
        if(both) {
            resetBoth()
        } else if(selectedMobType === 0){
            resetPassive()
        } else if (selectedMobType == 1) {
            resetAggressive()
        }
    }

    return (
        <>
            <SlotBar slots={config.slot_bars ?? createSlotBars()} onChange={handleSlotChange} />
            {/* DEBUG */}
            <Modal isShowing={debugModal.isShown} hide={debugModal.toggle} title={<h4>DEBUG</h4>} body={
                <ConfigTable>
                        <ConfigTableRow
                        label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(0);
                            mobsDebugModal.toggle();
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Agressive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(1);
                            mobsDebugModal.toggle();
                        }}>⚙️</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsDebugModal.isShown} hide={mobsDebugModal.toggle} title={(selectedMobType === 0)? <h4>Passive mob detection settings</h4> : <h4>Aggressive mob detection settings</h4>} body={
                <ConfigTable>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Colors" helpText="Monster's name color reference. Edit these values if you are sure what you are doing." />}
                            item={<ColorSelector value={(selectedMobType === 0)? config.passive_mobs_colors ?? defaultPassiveValues.passive_mobs_colors : config.aggressive_mobs_colors ?? defaultAggressiveValues.aggressive_mobs_colors} onChange={value => onChange?.((selectedMobType === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Tolerence" helpText="Monster's name color tolerence. Edit these values if you are sure what you are doing." />}
                            item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType === 0)? config.passive_tolerence ?? defaultPassiveValues.passive_tolerence : config.aggressive_tolerence ?? defaultAggressiveValues.aggressive_tolerence} onChange={value => onChange?.((selectedMobType === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                        />
                        <ConfigTableRow
                            label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                            item={<button onClick={()=>resetColorsRefs()}>Reset</button>}
                        />

                </ConfigTable>
            }/>

            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Avoid already attacked monster (experimental)" helpText="Check whether a mob is already attacked and avoid it if so. Must be disabled if you play in party" />}
                        item={<BooleanSlider value={config.prevent_already_attacked ?? false} onChange={value => onChange?.({ ...config, prevent_already_attacked: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Stay in Area" helpText="The bot will try to wait in the area and not move around too much." />}
                        item={<BooleanSlider value={config.stay_in_area ?? false} onChange={value => onChange?.({ ...config, stay_in_area: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Stop mob detection" helpText="Stop mob searching but keeps benefit of using the bot like item pickup, buffs, restoration, etc..." />}
                        item={<BooleanSlider value={config.is_stop_fighting ?? false} onChange={value => onChange?.({ ...config, is_stop_fighting: value })} />}
                    />


                    <ConfigTableRow
                        label={<ConfigLabel name="Debug settings" helpText="Change only if you know what you're doing !" />}
                        item={<button onClick={() => {
                            debugModal.toggle();
                        }}>⚙️</button>}
                    />

                </ConfigTable>
            </ConfigPanel>
        </>
    )
}

export default styled(FarmingConfig)`

`
