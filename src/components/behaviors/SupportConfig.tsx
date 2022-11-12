import { createSlotBars, SupportConfigModel } from "../../models/BotConfig"
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
import { DefaultValuesChecker } from "../utils/DefaultValuesChecker"
import NumericInput from "../config/NumericInput"
import TimeInput from "../config/TimeInput"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: SupportConfigModel,
    onChange: (config: SupportConfigModel) => void,
    running: boolean,
    isCurrentMode: boolean,
    botStopWatch: StopWatchValues | null,
    botState: string,


}

const SupportConfig = ({ className, info, config, onChange, running, isCurrentMode, botStopWatch, botState}: Props) => {
    const debugModal = useModal()
    const resetSlotYesNo = useModal(debugModal)
    const onDeathModal = useModal()

    const defaultValues = {
        'jump_cooldown': 5000,
    }

    DefaultValuesChecker(config, defaultValues, onChange)

    return (
        <>
            <SlotBar botMode="support" config={config} onChange={onChange} />
            <YesNoModal isShowing={resetSlotYesNo.isShown} hide={resetSlotYesNo.close}
                title={<h4>Confirm slot reset this action is irreversible</h4>}
                onYes={() => {
                    const newConfig = { ...config, slot_bars: createSlotBars() }
                    onChange(newConfig)
            }}/>
            <Modal isShowing={debugModal.isShown} hide={debugModal.close} title={<h4>DEBUG</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        label={<ConfigLabel name="Reset all slots" helpText="" />}
                        item={<button onClick={() => resetSlotYesNo.open()}>⚙️</button>}
                    />
                </ConfigTable>
            }/>
            <Modal isShowing={onDeathModal.isShown} hide={onDeathModal.close}
            title={<h4>On death behavior</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Disconnect" helpText="If enabled will automatically disconnect the dead character, otherwise we'll try to revive by pressing ENTER" />}
                        item={<BooleanSlider value={config.on_death_disconnect ?? true} onChange={value => onChange?.({ ...config, on_death_disconnect: value })} />}
                    />
                </ConfigTable>
            }/>
            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Jump cooldown" helpText="Time between two jumps If set to 0 the character will never jump." />}
                        item={<TimeInput value={config.jump_cooldown} onChange={value => onChange?.({...config, jump_cooldown: value})} />}
                    />
                </ConfigTable>
            </ConfigPanel>



            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: {botState}</div>
                    </div>
                    <div className="row">
                        <div>Botting time: {botStopWatch?.toString()}</div>
                    </div>
                    <div className="row">
                        <button className="btn sm" onClick={debugModal.open}>Debug ⚙️</button>
                        <button className="btn sm" onClick={onDeathModal.open}>On death</button>
                    </div>

                </div>
            )}
        </>
    )
}

export default styled(SupportConfig)`
`
