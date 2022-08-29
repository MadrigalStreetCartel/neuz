import React, { MouseEventHandler, useState } from 'react'
import styled from 'styled-components'
import IconMotionPickup from '../assets/icon_motion_pickup.png'
import { cooldownSlotTypes, SlotModel, SlotType, SLOT_SIZE_PX, thresholdSlotTypes, translateType } from '../models/BotConfig'
import ConfigLabel from './config/ConfigLabel'
import ConfigTableRow from './config/ConfigTableRow'
import NumericInput from './config/NumericInput'
import Select from 'react-select'
import Modal from './Modal'

type Props = {
    className?: string,
    isShowing: boolean,
    hide: () => void,
    index: number,
    slot?: SlotModel,
    onChange: (index:number, slot:SlotModel) => void,

}

const SlotModal = ({className, isShowing, hide, index, slot, onChange }: Props) => {
    const options = [
        { value: 'Unused', label: 'None' },
        { value: 'Food', label: '🍔 - Food' },
        { value: 'Pill', label: '💊 - Pill' },
        { value: 'MpRestorer', label: ' - MP' },
        { value: 'FpRestorer', label: ' - FP' },
        { value: 'PickupPet', label: '🐶 - PickupPet' },
        { value: 'PickupMotion', label: '- PickupMotion'  },
        { value: 'AttackSkill', label: '🗡️ - AttackSkill' },
        { value: 'BuffSkill', label: '🪄 - BuffSkill' },
        { value: 'Flying', label: '✈️ - Flying' },
    ]
    //const [selectedOption, setSelectedOption] = useState('None')
    if (slot) {
        //const symbolOrIcon = translateType(slot.slot_type)
        return(

            <Modal isShowing={isShowing} hide={hide} title={<h4>Slot {index} - {slot.slot_type}</h4>} body={
                <>
                    <ConfigTableRow
                        layout="v"
                        label={<ConfigLabel name="Type" helpText="Select action binded to current slot." />}
                        item={<div style={{width:'100%'}}><Select options={options} onChange={value => {slot.slot_type =  value?.value as SlotType || 'Unused';onChange(index, slot)}} defaultValue={options.find(x => x.value == slot.slot_type)}/></div>}
                    />

                    {cooldownSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Cooldown" helpText="Interval between to use." />}
                            item={<NumericInput unit="ms" value={slot.slot_cooldown ?? 1500} onChange={value => {slot.slot_cooldown = value;onChange(index, slot)}} />}
                        />
                    }

                    {thresholdSlotTypes.includes(slot.slot_type) &&
                        <ConfigTableRow
                            layout="v"
                            label={<ConfigLabel name="Threshold" helpText="Limit trigger value." />}
                            item={<NumericInput value={slot.slot_threshold ?? 50} onChange={value => {slot.slot_threshold = value;onChange(index, slot)}} />}
                        />
                    }
                </>
            }/>
    )
    }else {
        return(<></>)
    }

}


export default styled(SlotModal)`

`


