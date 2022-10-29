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
import { StopWatch } from "../utils/StopWatch"
import styled from "styled-components"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: SupportConfigModel,
    onChange: (config: SupportConfigModel) => void,
    running: boolean,
}

const SupportConfig = ({ className, info, config, onChange, running }: Props) => {
    const handleSlotChange = (slot_bar_index:number, slot_index:number, slot: SlotModel) => {
        const newConfig = { ...config, slot_bars: config.slot_bars ?? createSlotBars() }
        newConfig.slot_bars[slot_bar_index].slots[slot_index] = slot
        onChange(newConfig)
    }

    const debugModal = useModal()
    const resetSlotYesNo = useModal(debugModal)

    let botStopWatch = StopWatch()

    if(info?.is_running && info?.is_alive) {
        botStopWatch.start()
    }else {
        botStopWatch.stop()
    }

    return (
        <>
            <SlotBar botMode="support" config={config} onChange={handleSlotChange} />
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
        </>)


}

export default styled(SupportConfig)`
`
