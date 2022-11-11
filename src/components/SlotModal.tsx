import styled from 'styled-components'
import { cooldownSlotTypes, farmingSlotsBlacklist, SlotModel, SlotType, slotTypes, supportSlotsBlacklist, thresholdSlotTypes, translateDesc } from '../models/BotConfig'
import ConfigLabel from './config/ConfigLabel'
import ConfigTableRow from './config/ConfigTableRow'
import NumericInput from './config/NumericInput'
import Select from 'react-select'
import Modal from './Modal'
import ConfigTable from './config/ConfigTable'
import BooleanSlider from './config/BooleanSlider'
import TimeInput from './config/TimeInput'

type Props = {
    className?: string,
    isShowing: boolean,
    hide: () => void,
    index: number,
    slot?: SlotModel,
    onChange: (slot_bar_index:number, slot_index:number, slot:SlotModel) => void,
    barIndex: number,
    indexName: string,
    botMode: string,

}

const SlotModal = ({className, isShowing, hide, index, slot, onChange, barIndex, indexName, botMode}: Props) => {
    const blackList = botMode == "farming"? farmingSlotsBlacklist : supportSlotsBlacklist
    const options = slotTypes.map((type)=>{
        if (!blackList.includes(type))
        {
             return {value: type, label: translateDesc(type,"None")[1] }
        }
        return {}

    }).filter((item) => item.value != null )

    if (slot) {
        return(

            <Modal isShowing={isShowing} hide={hide} title={<h4>Slot F{barIndex + 1}-{indexName} - {slot.slot_type}</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Type" helpText="Select action binded to current slot." />}
                        item={<div style={{width:'100%', color: 'black'}}><Select options={options} onChange={value => {slot.slot_type =  value?.value as SlotType || 'Unused';onChange(barIndex, index, slot)}} defaultValue={options.find(x => x.value == slot.slot_type)}/></div>}
                    />

                    {cooldownSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Cooldown" helpText="Interval between two usage. Format must be : hh:mm:ss:mss" />}
                            item={<TimeInput value={slot.slot_cooldown} onChange={value => {slot.slot_cooldown = value;onChange(barIndex, index, slot)}} />}
                        />
                    }

                    {thresholdSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Threshold" helpText="Limit trigger value." />}
                            item={<NumericInput unit='%' value={slot.slot_threshold} onChange={value => {slot.slot_threshold = value;onChange(barIndex, index, slot)}} />}
                        />
                    }

                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Enabled" />}
                        item={<BooleanSlider value={slot.slot_enabled ?? true} onChange={value => {slot.slot_enabled = value;onChange(barIndex, index, slot)}} />}
                    />
                </ConfigTable>
            }/>
    )
    }else {
        return(<></>)
    }

}


export default styled(SlotModal)`
& img.type {
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: .25rem;
    border-radius: .25rem;
}
`


