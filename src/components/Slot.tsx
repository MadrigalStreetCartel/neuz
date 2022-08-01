import styled from 'styled-components'

import { SlotType, SlotTypes } from '../models/BotConfig'
import IconMotionPickup from '../assets/icon_motion_pickup.png'
import IconRefresher from '../assets/icon_refresher.png'
import IconVitalDrink from '../assets/icon_vitaldrink.png'

const SLOT_SIZE_PX = 40;

type Props = {
    className?: string,
    type: SlotType,
    index: number,
    isSelected:boolean,
    onChange?: (type: SlotType) => void,
    onSelected?:(val:number) =>void,
    hideIndex?:boolean,
}


const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return '🍔'
        case 'Pill': return '💊'
        case 'Refresher': return IconRefresher
        case 'VitalDrink': return IconVitalDrink
        case 'PickupPet': return '🐶'
        case 'PickupMotion': return IconMotionPickup
        case 'AttackSkill': return '🗡️'
        case 'BuffSkill': return '🪄'
        case 'Flying': return '✈️'
    }
}

const translateDesc = (type: SlotType) => {
    switch (type) {
        case 'Unused': return 'None'
        case 'Food': return 'Food'
        case 'Pill': return 'Pill'
        case 'Refresher': return 'MP'
        case 'VitalDrink': return 'FP'
        case 'PickupPet': return 'Pet'
        case 'PickupMotion': return 'Pickup'
        case 'AttackSkill': return 'Attack'
        case 'BuffSkill': return 'Buff'
        case 'Flying': return 'Board'
    }
}

const Slot = ({ className, type = 'Unused', index, onChange, onSelected,isSelected, hideIndex=false }: Props) => {
    const handleChange = () => {
        const nextType: SlotType = SlotTypes[(SlotTypes.indexOf(type) + 1) % SlotTypes.length];
        onChange?.(nextType)
    }
    const onClick = () => {
        if(onSelected != undefined){
            onSelected(index)
        }

    }

    const symbolOrIcon = translateType(type);
    const useIcon = symbolOrIcon?.startsWith('data:');

    return (
        <div className={className} onClick={onClick}>
            <div className={`${
                        isSelected
                            ? "selected panel"
                            : "panel"
                        }`}>


                {!hideIndex &&
                    <div className='index'>{index}</div>
                }

                {useIcon && (
                    <img className="type" src={symbolOrIcon} alt="Slot icon" />
                )}
                {!useIcon && (
                    <div style={(translateDesc(type) == "None")? {height:"100%",width:"100%"}: {}} className="type">{translateType(type)}</div>
                )}
                <div className="desc">{translateDesc(type)}</div>
            </div>
        </div>
    )
}

export default styled(Slot)`

    & .panel {
        display: flex;
        align-items: center;
        justify-content: center;
        width: ${SLOT_SIZE_PX}px;
        height: ${SLOT_SIZE_PX}px;
        background-color: hsla(0,0%,100%,.05);
        border: 1px solid hsl(48,58%,43%);
        border-radius: .25rem;
        position: relative;
        margin-top: .5rem;
        cursor: pointer;
    }
    & .selected {
        border: 1px solid hsl(0,100%,50%) !important;

    }

    &:first-child {
        order: 1;
    }

    & img.type:hover, div.type:hover {
        background-color: hsla(0,0%,100%,.1);
        border: 1px solid hsl(48,65%,50%);
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
        position: absolute;
        bottom: calc(-${SLOT_SIZE_PX / 3}px - 0.1rem);
        width: 100%;
        text-align: center;
        font-size: .75rem;
        color: hsl(48,75%,75%);
        text-shadow: 0 0 4px black;
    }

    & div.type {
        color: white;
        font-size: 1.5rem;
        width: 100%;
        height: 100%;
        text-align: center;
    }

    & img.type {
        width: 100%;
        height: 100%;
        object-fit: contain;
        padding: .25rem;
        border-radius: .25rem;
    }
`
