import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'

import SlotBar from '../SlotBar'
import { FarmingConfigModel, SlotBars, SlotModel } from '../../models/BotConfig'
import NumericInput from '../config/NumericInput'
import ColorSelector from '../config/ColorSelector'
import Modal from '../Modal'
import { useState } from 'react'
import { FrontendInfoModel } from '../../models/FrontendInfo'
import useModal from '../utils/UseModal'
import { StopWatch } from '../utils/StopWatch'
import YesNoModal from '../YesNoModal'

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
    const statsModal = useModal()
    const debugModal = useModal()
    const mobsNameDebugModal = useModal(debugModal)
    const mobsColorsDebugModal = useModal(mobsNameDebugModal)
    const resetSlotYesNo = useModal(debugModal)
    const obstacleAvoidanceDebugModal = useModal(debugModal)

    const [selectedMobType, setSelectedMobType] = useState(0)

    const defaultDetectionValues = [{passive_mobs_colors: [234, 234, 149], passive_tolerence: 5}, {aggressive_mobs_colors: [179, 23, 23], aggressive_tolerence: 10}]
    const resets = [() => onChange({...config, ...defaultDetectionValues[0] }), () => onChange({...config, ...defaultDetectionValues[1] })]
    const resetColorsRefs = (both?:boolean) => {
        const resetBoth = () => onChange({...config, ... resets[0], ... resets[1]})
        if(both) {
            resetBoth()
        } else {
            resets[selectedMobType]()
        }
    }
    let initial_circle_pattern_rotation_duration = false;
    if(!config.circle_pattern_rotation_duration && initial_circle_pattern_rotation_duration) {
        onChange({...config, circle_pattern_rotation_duration: 30})
        initial_circle_pattern_rotation_duration = true;
    }
    if(!config.passive_mobs_colors && !config.passive_tolerence) {
        resets[0]()
    }

    if(!config.aggressive_mobs_colors && !config.aggressive_tolerence) {
        resets[1]()
    }

    if(!config.obstacle_avoidance_cooldown && !config.obstacle_avoidance_max_try) {
        onChange({...config, obstacle_avoidance_cooldown: 5500, obstacle_avoidance_max_try: 3})
    }

    if(!config.min_mobs_name_width && !config.max_mobs_name_width) {
        onChange({...config, min_mobs_name_width: 15, max_mobs_name_width: 180})
    }

    // StopWatchs
    let botStopWatch = StopWatch(), searchMobStopWatch = StopWatch(), fightStopWatch = StopWatch()

    if(info?.is_running && info?.is_alive && !config.is_stop_fighting) {
        botStopWatch.start()
    }else {
        botStopWatch.stop()
    }

    if(info?.is_running && info?.is_alive && !config.is_stop_fighting && !info?.is_attacking) {
        searchMobStopWatch.start(true)
    }else {
        searchMobStopWatch.stop()
    }

    if(info?.is_running && info?.is_alive && !config.is_stop_fighting && info?.is_attacking) {
        fightStopWatch.start(true)
    }else {
        fightStopWatch.stop()
    }

    return (
        <>
            <SlotBar slots={config.slot_bars ?? createSlotBars()} onChange={handleSlotChange} />
            {/* DEBUG */}
            <YesNoModal isShowing={resetSlotYesNo.isShown} hide={resetSlotYesNo.close}
                title={<h4>Confirm slot reset this action is irreversible</h4>}
                onYes={() => {
                    const newConfig = { ...config, slot_bars: createSlotBars() }
                    onChange(newConfig)
            }}/>
            <Modal isShowing={debugModal.isShown} hide={debugModal.close} title={<h4>DEBUG</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Mobs detection settings" helpText="" />}
                        item={<button onClick={() => {
                            mobsNameDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Obstacle avoidance settings" helpText="" />}
                        item={<button onClick={() => {
                            obstacleAvoidanceDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset all slots" helpText="" />}
                        item={<button onClick={() => resetSlotYesNo.open()}>⚙️</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsColorsDebugModal.isShown} hide={mobsColorsDebugModal.close} title={(selectedMobType === 0)? <h4>Passive mob detection settings</h4> : <h4>Aggressive mob detection settings</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Colors" helpText="Monster's name color reference. Edit these values if you are sure what you are doing." />}
                        item={<ColorSelector value={(selectedMobType === 0)? config.passive_mobs_colors ?? [] : config.aggressive_mobs_colors ?? []} onChange={value => onChange?.((selectedMobType === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Tolerence" helpText="Monster's name color tolerence. Edit these values if you are sure what you are doing." />}
                        item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType === 0)? config.passive_tolerence ?? false : config.aggressive_tolerence ?? false} onChange={value => onChange?.((selectedMobType === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="" helpText="" />}
                        item={<button onClick={()=>resetColorsRefs()}>Reset</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={obstacleAvoidanceDebugModal.isShown} hide={obstacleAvoidanceDebugModal.close} title={<h4>Obstacle avoidance settings</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Obstacle avoidance enabled" helpText="" />}
                        item={<BooleanSlider value={config.obstacle_avoidance_enabled ?? true} onChange={value => onChange?.({ ...config, obstacle_avoidance_enabled: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Abort attack only for passive" helpText="" />}
                        item={<BooleanSlider value={config.obstacle_avoidance_only_passive ?? false} onChange={value => onChange?.({ ...config, obstacle_avoidance_only_passive: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance cooldown" helpText="Time before we try to move or escape if monster's HP doesn't change" />}
                        item={<NumericInput unit='ms' value={config.obstacle_avoidance_cooldown ?? false} onChange={value => onChange({...config, obstacle_avoidance_cooldown: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance max try" helpText="After this number of try it'll abort attack and search for another target" />}
                        item={<NumericInput unit='#' value={config.obstacle_avoidance_max_try ?? false} onChange={value => onChange({...config, obstacle_avoidance_max_try: value})} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsNameDebugModal.isShown} hide={mobsNameDebugModal.close} title={<h4>Mobs detection</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Passive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(0)
                            mobsColorsDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Agressive mob detection settings" helpText="" />}
                        item={<button onClick={() => {
                            setSelectedMobType(1)
                            mobsColorsDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Min mobs name width" helpText="" />}
                        item={<NumericInput unit='px' value={config.min_mobs_name_width ?? false} onChange={value => onChange({...config, min_mobs_name_width: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Max mobs name width" helpText="" />}
                        item={<NumericInput unit='px' value={config.max_mobs_name_width ?? false} onChange={value => onChange({...config, max_mobs_name_width: value})} />}
                    />
                </ConfigTable>
            }/>
            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Avoid attacked monster" helpText="Check whether a mob is already attacked and avoid it if so. Must be disabled if you play in party" />}
                        item={<BooleanSlider value={config.prevent_already_attacked ?? false} onChange={value => onChange?.({ ...config, prevent_already_attacked: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Circle pattern duration" helpText="The bot will try to move in a circle pattern to find target. Value of 0 will stay in place. Lower value to increase circle. Default : 30" />}
                        item={<NumericInput value={config.circle_pattern_rotation_duration ?? false} onChange={value => onChange?.({ ...config, circle_pattern_rotation_duration: value })} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Min HP percent to attack" helpText="Minimum required HP value to attack a monster (only for passive ones)" />}
                        item={<NumericInput unit='%' value={config.min_hp_attack ?? false} onChange={value => onChange({...config, min_hp_attack: value})} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Stop mob detection" helpText="Stop mob searching but keeps benefit of using the bot like item pickup, buffs, restoration, etc..." />}
                        item={<BooleanSlider value={config.is_stop_fighting ?? false} onChange={value => onChange?.({ ...config, is_stop_fighting: value })} />}
                    />
                </ConfigTable>
            </ConfigPanel>
            <Modal isShowing={statsModal.isShown} hide={statsModal.close} title={<h4>Stats</h4>} body={
                <div className="stats">
                    <div className="row">
                        <div>Last kill stats(approx): {info?.kill_min_avg}/min | {info?.kill_hour_avg}/hour | total : {info?.enemy_kill_count}</div>
                    </div>
                    <div className="row">
                        <div>Botting time: {botStopWatch.watch.hours}:{botStopWatch.watch.mins}:{botStopWatch.watch.secs}:{botStopWatch.watch.ms}</div>
                    </div>
                    <div className="row">
                        <div>Search time: {searchMobStopWatch.watch.hours}:{searchMobStopWatch.watch.mins}:{searchMobStopWatch.watch.secs}:{searchMobStopWatch.watch.ms}</div>
                    </div>
                    <div className="row">
                        <div>Fight time: {fightStopWatch.watch.hours}:{fightStopWatch.watch.mins}:{fightStopWatch.watch.secs}:{fightStopWatch.watch.ms}</div>
                    </div>
                </div>
            }/>
            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: {running? info.is_running? !info.is_alive? "dead" : config.is_stop_fighting? "manual" : info.is_attacking? "fighting" : "searching" : "ready" : "idle" }</div>
                    </div>
                    <button className="btn sm" onClick={statsModal.open}>Stats 📊</button>
                    <button className="btn sm" onClick={debugModal.open}>Debug ⚙️</button>
                </div>
            )}
        </>
    )
}

export default styled(FarmingConfig)`
`
