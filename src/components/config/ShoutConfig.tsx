import styled from 'styled-components'

import BooleanSlider from './BooleanSlider'
import ConfigLabel from './ConfigLabel'
import ConfigPanel from './ConfigPanel'
import ConfigRow from './ConfigRow'

import NumericInput from './NumericInput'
import StringList from './StringList'

import { ShoutConfigModel, SlotBarModel, SlotModel, SlotType } from '../../models/BotConfig'

type Props = {
    className?: string,
    config: ShoutConfigModel,
    onChange?: (config: ShoutConfigModel) => void,
}

const ShoutConfig = ({ className, config, onChange }: Props) => {
    return (
        <>
            <ConfigPanel>
                <ConfigRow reversed>
                    <ConfigLabel name="Messages" helpText="Add messages to the list to shout multiple messages. One message per line." />
                    <StringList messages={config.shout_messages ?? []} onChange={value => onChange?.({ ...config, shout_messages: value })} />
                </ConfigRow>
                <ConfigRow>
                    <ConfigLabel name="Interval" helpText="Interval between shouts in milliseconds." />
                    <NumericInput unit="ms" value={config.shout_interval ?? 30000} onChange={value => onChange?.({ ...config, shout_interval: value })} />
                </ConfigRow>
                {/* <ConfigRow>
                    <BooleanSlider value={config.shout_enabled ?? false} onChange={value => onChange?.({ ...config, shout_enabled: value })} />
                    <ConfigLabel name="Enable shouting" helpText="" />
                </ConfigRow> */}
            </ConfigPanel>
        </>
    )
}

export default styled(ShoutConfig)`
`
