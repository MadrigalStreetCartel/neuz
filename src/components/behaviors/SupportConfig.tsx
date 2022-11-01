import { createSlotBars, SlotModel, SupportConfigModel } from "../../models/BotConfig"
import { FrontendInfoModel } from "../../models/FrontendInfo"

import Modal from '../Modal'
import useModal from '../utils/UseModal'
import YesNoModal from '../YesNoModal'
import SlotBar from "../SlotBar"

import BooleanSlider from '../config/BooleanSlider'
import ConfigLabel from '../config/ConfigLabel'
import ConfigPanel from '../config/ConfigPanel'
import ConfigTable from '../config/ConfigTable'
import ConfigTableRow from '../config/ConfigTableRow'
import { useStopWatch } from "../utils/StopWatch"
import styled from "styled-components"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: SupportConfigModel,
    onChange: (config: SupportConfigModel) => void,
    running: boolean,
    isCurrentMode: boolean,
}

const SupportConfig = ({ className, info, config, onChange, running, isCurrentMode }: Props) => {
    const debugModal = useModal()
    const resetSlotYesNo = useModal(debugModal)

    let botStopWatch = useStopWatch()

    if(info) {
        botStopWatch.start(info?.is_running && info?.is_alive && isCurrentMode)
    }

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

            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: {running? info.is_running? !info.is_alive? "dead" : "healing" : "ready" : "idle"}</div>
                    </div>
                    <div className="row">
                        <div>Botting time: {botStopWatch.watch.hours}:{botStopWatch.watch.mins}:{botStopWatch.watch.secs}:{botStopWatch.watch.ms}</div>
                    </div>
                    <button className="btn sm" onClick={debugModal.open}>Debug ⚙️</button>
                </div>
            )}
        </>
    )
}

export default styled(SupportConfig)`
`
