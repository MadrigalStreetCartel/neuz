import styled from 'styled-components'
import { cooldownSlotTypes, SlotModel, SlotType, slotTypes, thresholdSlotTypes, translateDesc } from '../models/BotConfig'
import ConfigLabel from './config/ConfigLabel'
import ConfigTableRow from './config/ConfigTableRow'
import NumericInput from './config/NumericInput'
import Select from 'react-select'
import Modal from './Modal'
import ConfigTable from './config/ConfigTable'
import BooleanSlider from './config/BooleanSlider'

type Props = {
    className?: string,
    isShowing: boolean,
    hide: () => void,
    index: number,
    slot?: SlotModel,
    onChange: (slot_bar_index:number, slot_index:number, slot:SlotModel) => void,
    barIndex: number,
    indexName: string

}

const SlotModal = ({className, isShowing, hide, index, slot, onChange, barIndex, indexName }: Props) => {

    const options = slotTypes.map((type)=> ({value: type, label: translateDesc(type,"None") }) )

    //const [selectedOption, setSelectedOption] = useState('None')
    if (slot) {
        //const symbolOrIcon = translateType(slot.slot_type)
        return(

            <Modal isShowing={isShowing} hide={hide} title={<h4>Slot F{barIndex + 1}-{indexName} - {slot.slot_type}</h4>} body={
                <ConfigTable>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Type" helpText="Select action binded to current slot." />}
                        item={<div style={{width:'100%'}}><Select options={options} onChange={value => {slot.slot_type =  value?.value as SlotType || 'Unused';onChange(barIndex, index, slot)}} defaultValue={options.find(x => x.value == slot.slot_type)}/></div>}
                    />

                    {cooldownSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Cooldown" helpText="Interval between to use." />}
                            item={<NumericInput unit="ms" value={slot.slot_cooldown ?? false} onChange={value => {slot.slot_cooldown = value;onChange(barIndex, index, slot)}} />}
                        />
                    }

                    {thresholdSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Threshold" helpText="Limit trigger value." />}
                            item={<NumericInput unit='%' value={slot.slot_threshold ?? false} onChange={value => {slot.slot_threshold = value;onChange(barIndex, index, slot)}} />}
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


