import styled from 'styled-components'

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'
import NumericInput from '../config/NumericInput'
import ColorSelector from '../config/ColorSelector'

import SlotBar from '../SlotBar'
import { createSlotBars, FarmingConfigModel } from '../../models/BotConfig'
import { useEffect, useRef, useState } from 'react'
import { FrontendInfoModel } from '../../models/FrontendInfo'
import Modal from '../Modal'
import useModal from '../utils/UseModal'
import YesNoModal from '../YesNoModal'
import TimeInput from '../config/TimeInput'
import useDefaultValue from '../utils/useDefaultValues'
import { Time } from '../utils/Time'

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: FarmingConfigModel,
    onChange: (config: FarmingConfigModel) => void,
    botStopWatch: Time | null,
    botState: string,
}

const FarmingConfig = ({ className, info, config, onChange, botStopWatch, botState }: Props) => {
    const statsModal = useModal()
    const settingsModal = useModal()

    // StopWatchs
    const searchMobStopWatch = new Time(info?.last_search_duration ?? 0).toString(),
    fightStopWatch = new Time(info?.last_search_duration ?? 0).toString()

    const globalKPM = ((info?.enemy_kill_count?? 0) / Math.round(Number(botStopWatch?.time ?? 0) / 60000)).toFixed(2)
    const globalKPH = (Number(globalKPM) * 60).toFixed(2)

    return (
        <div className={className}>
            <SlotBar botMode="farming" config={config} onChange={onChange} />
            <Modal isShowing={settingsModal.isShown} hide={settingsModal.close} title={<h4>Settings</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Actively avoid aggressives" helpText="Will try to run the opposite way of an aggressive target" />}
                        item={<BooleanSlider value={config.active_avoid_aggressive ?? false} onChange={value => onChange?.({ ...config, active_avoid_aggressive: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Avoid attacked monster" helpText="Check whether a mob is already attacked and avoid it if so. Must be disabled if you play in party" />}
                        item={<BooleanSlider value={config.prevent_already_attacked ?? true} onChange={value => onChange?.({ ...config, prevent_already_attacked: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance max try" helpText="After this number of try it'll abort attack and search for another target" />}
                        item={<NumericInput unit='#' value={config.obstacle_avoidance_max_try} onChange={value => onChange({...config, obstacle_avoidance_max_try: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Circle pattern duration" helpText="The bot will try to move in a circle pattern to find targets. Value of 0 will stay in place. Lower the value to increase circle size. Default : 30" />}
                        item={<NumericInput value={config.circle_pattern_rotation_duration} onChange={value => onChange?.({ ...config, circle_pattern_rotation_duration: value })} />}
                    />
                </ConfigTable>
            }/>

            <Modal isShowing={statsModal.isShown} hide={statsModal.close}
            title={<h4>Stats - State: { botState }</h4>} body={
                <div className="stats">
                    <div className="row">
                        <div>Kills : {info?.enemy_kill_count}</div>
                    </div>
                    <div className="row">
                        <div>Botting time: {botStopWatch?.toString()}</div>
                    </div>
                    <div className="row">
                        <div>Last search time: {searchMobStopWatch}</div>
                    </div>
                    <div className="row">
                        <div>Last fight time: {fightStopWatch}</div>
                    </div>
                    <div className="row">
                        <div>Last kill stats(approx): {info?.kill_min_avg}/min | {info?.kill_hour_avg}/hour</div>
                    </div>
                    <div className="row">
                        <div>Global kills stats(approx): {globalKPM === "NaN" || globalKPM === "Infinity" ? 0 : globalKPM}/min
                        | {globalKPH === "NaN" || globalKPH === "Infinity" ? 0 : globalKPH}/hour</div>
                    </div>
                </div>
            }/>
            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: { botState }</div>
                    </div>
                    <div className="row">
                        <div>Target's detection mode: { config.is_manual_targetting? "🛑" : "✅" }</div>
                    </div>
                    <button className="btn sm"
                            onClick={e => onChange?.({ ...config, is_manual_targetting: !config.is_manual_targetting })} >
                            Detection 🎯
                    </button>
                    <div className="row">
                        <button className="btn sm" onClick={statsModal.open}>Stats 📊</button>
                        <button className="btn sm" onClick={settingsModal.open}>Settings ⚙️</button>
                    </div>
                </div>
            )}
        </div>
    )
}

export default styled(FarmingConfig)`
    & .modalCenteredContent {
        text-align: center;
    }
`
