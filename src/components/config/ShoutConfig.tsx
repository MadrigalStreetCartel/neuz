import styled from 'styled-components'

import BooleanSlider from './BooleanSlider'
import ConfigLabel from './ConfigLabel'
import ConfigPanel from './ConfigPanel'
import ConfigRow from './ConfigRow'

import StringInput from './StringInput'
import NumericInput from './NumericInput'

import { ShoutConfigModel, SlotBarModel, SlotModel, SlotType } from '../../models/BotConfig'
import InputList from './InputList'

type Props = {
    className?: string,
    config: ShoutConfigModel,
    onChange?: (config: ShoutConfigModel) => void,
}

const createMessageList = () => (
    [...new Array(10)].map(_ => "") 
)

const ShoutConfig = ({ className, config, onChange }: Props) => {
    return (
        <>
            <ConfigPanel>
                    <ConfigLabel name="Message" helpText="Add messages to the list to shout multiple messages." />
                    <InputList value={config.shout_message ?? []} onChange={value => onChange?.({ ...config, shout_message: value })} />                  
                <ConfigRow>
                    <NumericInput value={config.shout_interval ?? 60} onChange={value => onChange?.({ ...config, shout_interval: value })} />
                    <ConfigLabel name="Interval" helpText="Interval between two shoutouts value need to be in seconds" />
                </ConfigRow>
                <ConfigRow>
                    <BooleanSlider value={config.shout_enabled ?? false} onChange={value => onChange?.({ ...config, shout_enabled: value })} />
                    <ConfigLabel name="Enable shouting" helpText="" />
                </ConfigRow>

            </ConfigPanel>
        </>
    )
}

export default styled(ShoutConfig)`
`