import { BotConfigModel, createSlotBars, SupportConfigModel } from "../../models/BotConfig"
import { FrontendInfoModel } from "../../models/FrontendInfo"

import Modal from '../Modal'
import useModal from '../utils/UseModal'
import YesNoModal from '../YesNoModal'
import SlotBar from "../SlotBar"
import { StopWatchValues, useStopWatch } from "../utils/StopWatch"

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'
import styled from "styled-components"
import NumericInput from "../config/NumericInput"
import TimeInput from "../config/TimeInput"
import { useEffect, useRef, useState } from "react"
import ColorSelector from "../config/ColorSelector"
import useDefaultValue from "../utils/useDefaultValues"
import StringList from "../config/StringList"
import WhiteList from "../config/WhiteList"
import TextInput from "../config/TextInput"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: BotConfigModel,
    onChange: (config: BotConfigModel) => void,
}

const Settings = ({ className, info, config, onChange}: Props) => {
    const mobsDetectionModal = useModal()
    const mobsColorsDebugModal = useModal(mobsDetectionModal)
    const autoDisconnectModal = useModal()
    const miscModal = useModal()
    const mobsAvoidanceModal = useModal()
    const addWhitelistModal = useModal(mobsAvoidanceModal)

    const resetSlotModal = useModal()
    const resetSlotYesNo = useModal(resetSlotModal)

    const resetSlotMode = useRef("farming")

    const addWhitelistWidth = useRef(0)
    const addWhitelistHeight = useRef(0)
    const addWhitelistName = useRef("")

    useEffect(() => {
        addWhitelistWidth.current = info?.last_mob_width ?? 0
        addWhitelistHeight.current = info?.last_mob_height ?? 0
    }, [info?.last_mob_height, info?.last_mob_width])


    const selectedMobType = useRef(0)

    const settingsReseter = useDefaultValue(config, onChange)

    const colorsRefResetter = [
        () => settingsReseter.reset(['passive_mobs_colors', 'passive_tolerence']),
        () => settingsReseter.reset(['aggressive_mobs_colors', 'aggressive_tolerence'])
    ]

    const setIsStopFighting = (value: boolean) => {
        onChange?.({ ...config, farming_config: {...config.farming_config, is_stop_fighting: value} })
    }

    return (
        <div className={className}>
            <Modal isShowing={mobsColorsDebugModal.isShown} hide={mobsColorsDebugModal.close} title={(selectedMobType.current === 0)? <h4>Passive mob detection</h4> : <h4>Aggressive mob detection</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Colors" helpText="Monster's name color reference. Edit these values if you are sure what you are doing." />}
                        item={<ColorSelector value={(selectedMobType.current === 0)? config.passive_mobs_colors ?? [] : config.aggressive_mobs_colors ?? []} onChange={value => onChange?.((selectedMobType.current === 0)?{ ...config, passive_mobs_colors: value}: { ...config, aggressive_mobs_colors: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Tolerence" helpText="Monster's name color tolerence. Edit these values if you are sure what you are doing." />}
                        item={<NumericInput min={0} max={255} unit="#" value={(selectedMobType.current === 0)? config.passive_tolerence : config.aggressive_tolerence} onChange={value => onChange?.((selectedMobType.current === 0)? { ...config, passive_tolerence: value } : { ...config, aggressive_tolerence: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="" helpText="" />}
                        item={<button className="btn sm" onClick={()=>colorsRefResetter[selectedMobType.current]()}>Reset</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={mobsDetectionModal.isShown} hide={mobsDetectionModal.close} title={<h4>Mobs detection</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Passive mob detection" helpText="" />}
                        item={<button onClick={() => {
                            selectedMobType.current = 0
                            mobsColorsDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Agressive mob detection" helpText="" />}
                        item={<button onClick={() => {
                            selectedMobType.current = 1
                            mobsColorsDebugModal.open()
                        }}>⚙️</button>}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Min mobs name width" helpText="" />}
                        item={<NumericInput unit='px' value={config.min_mobs_name_width} onChange={value => onChange({...config, min_mobs_name_width: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Max mobs name width" helpText="" />}
                        item={<NumericInput unit='px' value={config.max_mobs_name_width} onChange={value => onChange({...config, max_mobs_name_width: value})} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={autoDisconnectModal.isShown} hide={autoDisconnectModal.close}
                title={<h4>Auto disconnect</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Inactivity timeout" helpText="After this time character will disconnect if nothing happen" />}
                        item={<TimeInput value={config.inactivity_timeout} onChange={value => onChange({...config, inactivity_timeout: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Disconnect on death" helpText="Enable will automatically disconnect on death, otherwise we'll wait for revive and press ENTER" />}
                        item={<BooleanSlider value={config.on_death_disconnect ?? false} onChange={value => onChange?.({ ...config, on_death_disconnect: value })} />}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={miscModal.isShown} hide={miscModal.close} title={<h4>Misc</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Interval between buffs" helpText="" />}
                        item={<TimeInput value={config.interval_between_buffs} onChange={value => onChange({...config, interval_between_buffs: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Obstacle avoidance cooldown" helpText="Time before we try to move or escape if monster's HP doesn't change" />}
                        item={<TimeInput value={config.obstacle_avoidance_cooldown} onChange={value => onChange({...config, obstacle_avoidance_cooldown: value})} />}
                    />
                </ConfigTable>
            }/>

            <YesNoModal isShowing={resetSlotYesNo.isShown} hide={resetSlotYesNo.close}
                title={<h4>Confirm slot reset for {resetSlotMode.current} this action is irreversible</h4>}
                onYes={() => {
                if (resetSlotMode.current === "farming") {
                    const newConfig = { ...config, farming_config: { ...config.farming_config, slot_bars: createSlotBars()}  }
                    onChange(newConfig)
                } else if (resetSlotMode.current === "support") {
                    const newConfig = { ...config, farming_config: { ...config.support_config, slot_bars: createSlotBars()}  }
                    onChange(newConfig)
                }
            }}/>

            <Modal isShowing={resetSlotModal.isShown} hide={resetSlotModal.close} title={<h4>Reset slots</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset farming's slots" helpText="" />}
                        item={<button className="btn sm" onClick={() => {resetSlotMode.current = "farming" ;resetSlotYesNo.open()}}>Reset</button>}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset support's slots" helpText="" />}
                        item={<button className="btn sm" onClick={() => {resetSlotMode.current = "support" ;resetSlotYesNo.open()}}>Reset</button>}
                    />
                </ConfigTable>
            }/>

            <Modal isShowing={addWhitelistModal.isShown} hide={addWhitelistModal.close} title={<h4>Adding to whitelist</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Width" helpText="" />}
                        item={<NumericInput unit='px' value={addWhitelistWidth.current ?? 0} onChange={value => addWhitelistWidth.current = value} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Height" helpText="" />}
                        item={<NumericInput unit='px' value={addWhitelistHeight.current ?? 0} onChange={value => addWhitelistHeight.current = value} />}
                    />
                    <ConfigTableRow
                        label={<ConfigLabel name="Name" helpText="" />}
                        item={<TextInput value={addWhitelistName.current ?? ""} onChange={value => addWhitelistName.current = value} />}
                    />
                    <button className="btn sm" onClick={() => {onChange({...config, whitelist: [...config.whitelist ?? [], [addWhitelistWidth.current, addWhitelistHeight.current, addWhitelistName.current] ]});addWhitelistModal.close(); setIsStopFighting(false); addWhitelistName.current = ""}}>Add</button>
                </ConfigTable>
            }/>
            <Modal isShowing={mobsAvoidanceModal.isShown} hide={mobsAvoidanceModal.close}
                title={<h4>Mobs whitelist</h4>} body={
                <ConfigTable>
                    <h4 style={{textAlign: "center"}}>Last targetted mob bounds:<br />width: {info?.last_mob_width ?? 0}px height: {info?.last_mob_height ?? 0}px</h4>
                    <button className="btn m" onClick={() => {setIsStopFighting(true); addWhitelistModal.open()}}>Add</button>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="White list" helpText="" />}
                        item={<WhiteList whitelist={config.whitelist ?? []} onChange={ value => onChange({...config, whitelist: value})} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Enable white list" helpText="Will target all mobs if disabled" />}
                        item={<BooleanSlider value={config.whitelist_enabled ?? false} onChange={value => onChange?.({ ...config, whitelist_enabled: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Mob detection" helpText="Just the same that farming page but helps a lot here" />}
                        item={<BooleanSlider value={!config.farming_config?.is_stop_fighting ?? false} onChange={value => setIsStopFighting(!value)} />}
                    />
                </ConfigTable>
            }/>

            <div className="info">
                <div className="row">
                    <button className="btn sm" onClick={miscModal.open}>Misc</button>
                </div>
                <div className="row">
                    <button className="btn sm" onClick={mobsDetectionModal.open}>Mobs detection</button>
                </div>
                <div className="row">
                    <button className="btn sm" onClick={autoDisconnectModal.open}>Auto disconnect</button>
                </div>
                <div className="row">
                    <button className="btn sm" onClick={mobsAvoidanceModal.open}>Mobs whitelist</button>
                </div>
                <div className="row">
                    <button className="btn sm" onClick={resetSlotModal.open}>Reset slots</button>
                </div>
            </div>
        </div>
    )
}

export default styled(Settings)`
`
