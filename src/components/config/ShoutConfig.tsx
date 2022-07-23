import styled from 'styled-components'

import BooleanSlider from './BooleanSlider'
import ConfigLabel from './ConfigLabel'
import ConfigPanel from './ConfigPanel'

import NumericInput from './NumericInput'
import StringList from './StringList'

import { ShoutConfigModel, SlotBarModel, SlotModel, SlotType } from '../../models/BotConfig'
import ConfigTable from './ConfigTable'
import ConfigTableRow from './ConfigTableRow'

type Props = {
    config: ShoutConfigModel,
    onChange?: (config: ShoutConfigModel) => void,
}

const ShoutConfig = ({ config, onChange }: Props) => {
    return (
        <>
            <ConfigPanel>
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Messages" helpText="Add messages to the list to shout multiple messages. One message per line." />}
                        item={<StringList messages={config.shout_messages ?? []} onChange={value => onChange?.({ ...config, shout_messages: value })} />}
                    />
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Interval" helpText="Interval between shouts in milliseconds." />}
                        item={<NumericInput unit="ms" value={config.shout_interval ?? 30000} onChange={value => onChange?.({ ...config, shout_interval: value })} />}
                    />
                </ConfigTable>
                {/* <ConfigRow>
                    <BooleanSlider value={config.shout_enabled ?? false} onChange={value => onChange?.({ ...config, shout_enabled: value })} />
                    <ConfigLabel name="Enable shouting" helpText="" />
                </ConfigRow> */}
            </ConfigPanel>
        </>
    )
}

export default ShoutConfig;
