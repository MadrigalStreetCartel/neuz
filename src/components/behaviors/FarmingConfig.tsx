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
import { useState } from 'react'
import { FrontendInfoModel } from '../../models/FrontendInfo'
import useModal from '../utils/UseModal'
import { StopWatch } from '../utils/StopWatch'

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: FarmingConfigModel,
    onChange: (config: FarmingConfigModel) => void,
    running: boolean,
}

const createSlotBars = () => (
    [...new Array(9)].map(_ => ({slots:[...new Array(10)].map(_ => ({ slot_type: 'Unused', slot_enabled: false } as SlotModel))})) as SlotBars
)

const FarmingConfig = ({ className, info, config, onChange, running }: Props) => {
    const handleSlotChange = (slot_bar_index:number, slot_index:number, slot: SlotModel) => {
        const newConfig = { ...config, slot_bars: config.slot_bars ?? createSlotBars() }
        newConfig.slot_bars[slot_bar_index].slots[slot_index] = slot
        onChange(newConfig)
    }
    const debugModal = useModal();
    const mobsDebugModal = useModal(debugModal);
    const mobsAvoidanceDebugModal = useModal(debugModal);

    const [debugModeCount, setDebugModeCount] = useState(0)

    const [selectedMobType, setSelectedMobType] = useState(0)
    const defaultDetectionValues = [{passive_mobs_colors: [234, 234, 149], passive_tolerence: 5}, {aggressive_mobs_colors: [179, 23, 23], aggressive_tolerence: 10}]

    const resetColorsRefs = (both?:boolean) => {

        const resets = [() => onChange({...config, ...defaultDetectionValues[0] }), () => onChange({...config, ...defaultDetectionValues[1] })]
        const resetBoth = () => onChange({...config, ... resets[0], ... resets[1]})
        if(both) {
            resetBoth()
        } else {
            resets[selectedMobType]()
        }
    }

    let BotStopWatch = StopWatch((info?.is_running ?? false) && (info?.is_alive ?? false) && (!config.is_stop_fighting ?? false));

    return (
        <>
            <SlotBar slots={config.slot_bars ?? createSlotBars()} onChange={handleSlotChange} />
            {/* DEBUG */}
            <Modal isShowing={debugModal.isShown} hide={debugModal.close} title={<h4>DEBUG</h4>} body={
                <ConfigTable>
                        <ConfigTableRow
                        label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(0);
                            mobsDebugModal.open();
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Agressive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(1);
                            mobsDebugModal.open();
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Obstacles avoidance" helpText="" />}
                        item={<button onClick={() => {
                            mobsAvoidanceDebugModal.open();
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset all slots" helpText="" />}
                        item={<button onClick={() => {
                            const newConfig = { ...config, slot_bars: createSlotBars() }
                            onChange(newConfig)
                        }}>⚙️</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsDebugModal.isShown} hide={mobsDebugModal.close} title={(selectedMobType === 0)? <h4>Passive mob detection settings</h4> : <h4>Aggressive mob detection settings</h4>} body={
                <ConfigTable>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Colors" helpText="Monster's name color reference. Edit these values if you are sure what you are doing." />}
                            item={<ColorSelector value={(selectedMobType === 0)? config.passive_mobs_colors ?? defaultDetectionValues[0].passive_mobs_colors ?? [] : config.aggressive_mobs_colors ?? defaultDetectionValues[1].aggressive_mobs_colors ?? []} onChange={value => onChange?.((selectedMobType === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Tolerence" helpText="Monster's name color tolerence. Edit these values if you are sure what you are doing." />}
                            item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType === 0)? config.passive_tolerence ?? defaultDetectionValues[0].passive_tolerence ?? 0 : config.aggressive_tolerence ?? defaultDetectionValues[1].aggressive_tolerence ?? 0} onChange={value => onChange?.((selectedMobType === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                        />
                        <ConfigTableRow
                            label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                            item={<button onClick={()=>resetColorsRefs()}>Reset</button>}
                        />

                </ConfigTable>
            }/>
            <Modal isShowing={mobsAvoidanceDebugModal.isShown} hide={mobsAvoidanceDebugModal.close} title={<h4>Obstacle avoidance settings</h4>} body={
                <ConfigTable>
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Cooldown" helpText="When monster don't loose HP within this cooldown" />}
                            item={<NumericInput unit="ms" value={config.obstacle_avoidance_cooldown ?? 3500} onChange={value => onChange?.({ ...config, obstacle_avoidance_cooldown: value })} />}
                        />
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Maximum avoidance" helpText="After this amount of try it will cancel and search for another target" />}
                            item={<NumericInput unit="#" value={config.obstacle_avoidance_max_count ?? 2} onChange={value => onChange?.({ ...config, obstacle_avoidance_max_count: value })} />}
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

                    {debugModeCount >= 3 &&
                        <ConfigTableRow
                            label={<ConfigLabel name="Debug settings" helpText="Change only if you know what you're doing !" />}
                            item={<button onClick={() => {
                                debugModal.open();
                            }}>⚙️</button>}
                        />
                    }


                </ConfigTable>

            </ConfigPanel>
            {info && (
                    <div className="info" onClick={() => setDebugModeCount(debugModeCount >= 3 ? 0 : debugModeCount + 1) }>
                        <div className="row">
                            <div>State: {running? info.is_running? !info.is_alive? "dead" : config.is_stop_fighting? "manual" : info.is_attacking? "fighting" : "searching" : "ready" : "idle" }</div>
                        </div>
                        <div className="row">
                            <div>Kills stats(approx): {info.kill_min_avg}/min | {info.kill_hour_avg}/hour | total : {info.enemy_kill_count}</div>
                        </div>
                        <div className="row">
                            <div>Botting time : {BotStopWatch[0]}:{BotStopWatch[1]}:{BotStopWatch[2]}</div>
                        </div>

                    </div>
                )}
        </>
    )
}

export default styled(FarmingConfig)`
`
