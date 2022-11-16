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
import NumericInput from "../config/NumericInput"
import TimeInput from "../config/TimeInput"

type Props = {
    className?: string,
    info: FrontendInfoModel | null,
    config: SupportConfigModel,
    onChange: (config: SupportConfigModel) => void,
    botStopWatch: StopWatchValues | null,
    botState: string,


}

const SupportConfig = ({ className, info, config, onChange, botStopWatch, botState}: Props) => {
    return (
        <div className={className}>
            <SlotBar botMode="support" config={config} onChange={onChange} />
            {info && (
                <div className="info">
                    <div className="row">
                        <div>State: {botState}</div>
                    </div>
                    <div className="row">
                        <div>Botting time: {botStopWatch?.toString()}</div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default styled(SupportConfig)`
`
