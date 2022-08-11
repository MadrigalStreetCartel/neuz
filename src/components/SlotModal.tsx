import React, { MouseEventHandler, useState } from 'react'
import styled from 'styled-components'
import IconMotionPickup from '../assets/icon_motion_pickup.png'
import { cooldownSlotTypes, SlotModel, SlotType, SLOT_SIZE_PX, thresholdSlotTypes, translateType } from '../models/BotConfig'
import ConfigLabel from './config/ConfigLabel'
import ConfigTableRow from './config/ConfigTableRow'
import NumericInput from './config/NumericInput'
import Select from 'react-select'

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
        { value: 'PickupMotion', label:<div><img style={{maxWidth:'22px'}} className="type" src={IconMotionPickup} alt="Slot icon" /> - PickupMotion</div>  },
        { value: 'AttackSkill', label: '🗡️ - AttackSkill' },
        { value: 'BuffSkill', label: '🪄 - BuffSkill' },
        { value: 'Flying', label: '✈️ - Flying' },
    ]
    const [selectedOption, setSelectedOption] = useState('None')
    if (isShowing && slot) {
        const symbolOrIcon = translateType(slot.slot_type)
        const useIcon = symbolOrIcon.startsWith('data:');
        return(
            <div className={className} >
                <div className="modal-overlay">
                    <div className="modal-wrapper" onMouseDown={event => { if (event.target === event.currentTarget)  hide()}}>
                    <div className="modal">
                        <div className="modal-header">
                        <h4>Slot {index} - {slot.slot_type}</h4>
                        <button
                            type="button"
                            className="modal-close-button"
                            onClick={hide}
                        >
                            <span style={{color:"white"}}>&times;</span>
                        </button>
                        </div>

                        <div className="modal-body">

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

                        </div>
                    </div>
                    </div>
                </div>
            </div>
    )
    }else {
        return(<></>)
    }

}


export default styled(SlotModal)`
position: absolute;
& .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 1040;
    background-color: rgba(0, 0, 0, 0.2);
}
&.select-slot {
    color: inherit;
    color: black !important;
    display: inline-block;
    fontSize: 12;
    fontStyle: italic;
    marginTop: 1em;
}
& .modal-wrapper {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 1050;
    width: 100%;
    height: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    outline: 0;
    display: flex;
    align-items: center;
}

& .modal {
    z-index: 100;
    background: #fff;
    position: relative;
    margin: auto;
    border-radius: 5px;
    max-width: 500px;
    height:auto;
    width: 80%;
    padding: 1rem;
    background: hsla(0,0%,0%,.75);
    backdrop-filter: blur(.9rem);
}

& .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: white;

}

& .modal-close-button {
    font-size: 1.4rem;
    font-weight: 700;
    color: #000;
    cursor: pointer;
    border: none;
    background: transparent;
}

&.modal-body {
    display: flex;
    flex-wrap: wrap;
}


& .slot {
    text-align: center;
    background-color: hsla(0,0%,100%,.05);
}

& .index {
    position: absolute;
    top: calc(-${SLOT_SIZE_PX / 3}px - 0.2rem);
    width: 100%;
    text-align: center;
    font-size: 0.75rem;
    color: hsl(48,75%,75%);
    text-shadow: 0 0 4px black;
}

& .desc {
    width: 100%;
    text-align: center;
    font-size: 1rem;
    }

& div.type {
    color: white;
    font-size: 1.5rem;
}

& img.type {
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: .25rem;
    border-radius: .25rem;
}

`


