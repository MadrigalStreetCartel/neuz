import styled from 'styled-components'

import { SlotModel, SlotType } from '../models/BotConfig'
import IconMotionPickup from '../assets/icon_motion_pickup.png'
import { BsFillGearFill } from 'react-icons/bs'

const SLOT_SIZE_PX = 40;

type Props = {
    className?: string,
    type: SlotType,
    index: number,
    onChange?: (type: SlotType) => void,
    toggleSlotModal: () => void,
}

const types: SlotType[] = ['Unused', 'Food', 'Pill', 'PickupPet', 'PickupMotion', 'AttackSkill', 'BuffSkill', 'Flying']

const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return '🍔'
        case 'Pill': return '💊'
        case 'PickupPet': return '🐶'
        case 'PickupMotion': return IconMotionPickup
        case 'AttackSkill': return '🗡️'
        case 'BuffSkill': return '🪄'
        case 'Flying': return '✈️'
    }
}

const translateDesc = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return 'Food'
        case 'Pill': return 'Pill'
        case 'PickupPet': return 'Pet'
        case 'PickupMotion': return 'Pickup'
        case 'AttackSkill': return 'Attack'
        case 'BuffSkill': return 'Buff'
        case 'Flying': return 'Board'
    }
}

const Slot = ({ className, type = 'Unused', index, onChange, toggleSlotModal }: Props) => {
    const handleChange = () => {
        const nextType: SlotType = types[(types.indexOf(type) + 1) % types.length];
        onChange?.(nextType)
    }
    const symbolOrIcon = translateType(type)
    const useIcon = symbolOrIcon.startsWith('data:');

    return (

        <div className={className} >
            <div className="index" onClick={toggleSlotModal}>{index} <BsFillGearFill/></div>
            <div className='slot' onClick={handleChange}>
                {useIcon && (
                    <img className="type" src={symbolOrIcon} alt="Slot icon" />
                )}
                {!useIcon && (
                    <div className="type">{translateType(type)}</div>
                )}
                <div className="desc">{translateDesc(type)}</div>
            </div>
        </div>
    )
}

export default styled(Slot)`
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

    &:first-child {
        order: 1;
    }

    &:hover {
        background-color: hsla(0,0%,100%,.1);
        border: 1px solid hsl(48,65%,50%);
    }

    & .slot {
        height:100%;
        width: 100%;
        text-align: center;
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
    }

    & img.type {
        width: 100%;
        height: 100%;
        object-fit: contain;
        padding: .25rem;
        border-radius: .25rem;
    }
`
