import styled from 'styled-components'

import { SlotType } from '../models/BotConfig'

const SLOT_SIZE_PX = 40;

type Props = {
    className?: string,
    type: SlotType,
    index: number,
    onChange?: (type: SlotType) => void,
}

const types: SlotType[] = ['Unused', 'Food', 'PickupPet', 'AttackSkill', 'BuffSkill', 'Flying']

const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return '🍔'
        case 'PickupPet': return '🐶'
        case 'AttackSkill': return '🗡️'
        case 'BuffSkill': return '🪄' 
        case 'Flying': return '✈️'
    }
}

const Slot = ({ className, type = 'Unused', index, onChange }: Props) => {
    const handleChange = () => {
        const nextType: SlotType = types[(types.indexOf(type) + 1) % types.length];
        onChange?.(nextType)
    }

    return (
        <div className={className} onClick={handleChange}>
            <div className="index">{index}</div>
            <div className="type">{translateType(type)}</div>
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

    & .index {
        position: absolute;
        top: -${SLOT_SIZE_PX / 3}px;
        width: 100%;
        text-align: center;
        font-size: 1rem;
        color: hsl(48,75%,75%);
        text-shadow: 0 0 4px black;
    }

    & .type {
        color: white;
        font-size: 1.5rem;
    }
`