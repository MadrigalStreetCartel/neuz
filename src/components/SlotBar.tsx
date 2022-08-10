import styled from 'styled-components'

import Slot from './Slot'

import { SlotType, SlotBarModel, SlotModel } from '../models/BotConfig'
import SlotModal from './SlotModal'
import useModal from './UseModal'
import { useState } from 'react'

type Props = {
    className?: string,
    slots: SlotBarModel,
    onChange?: (type: SlotType, index: number) => void,
    updateSlot: (index:number, slot:SlotModel) => void,
}

const SlotBar = ({ className, slots, onChange, updateSlot }: Props) => {
    const { isShowing, toggle } = useModal();

    const [currentSlotId, setCurrentSlotId] = useState(-1)

    const toogleSlot = (id: number) => {
        setCurrentSlotId(id)
        toggle()
    }

    return (
        <>
            <SlotModal isShowing={isShowing} hide={toggle} index={currentSlotId} slot={slots[currentSlotId]} updateSlot={updateSlot}/>
            <div className={className}>
                {slots.map((slot, index) => (
                    <Slot key={index} type={slot.slot_type} index={index} onChange={type => null/*type => onChange?.(type, index)*/} toggleSlotModal={() => toogleSlot(index)} />
                ))}
            </div>
        </>
    )
}

export default styled(SlotBar)`
    display: flex;
    align-items: center;
    justify-content: center;
    gap: .25rem;
    background: hsla(0,0%,0%,.75);
    backdrop-filter: blur(.5rem);
    padding: .5rem 2.5rem 1rem 2.5rem;
    border-radius: 100rem;
    border: 1px solid hsl(48,58%,43%);

`
