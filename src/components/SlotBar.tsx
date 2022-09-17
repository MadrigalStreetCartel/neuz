import styled from 'styled-components'

import Slot from './Slot'

import { SlotBarHolder, SlotBars, SlotModel } from '../models/BotConfig'
import SlotModal from './SlotModal'
import useModal from './UseModal'
import { useState } from 'react'

type Props = {
    className?: string,
    slots: SlotBars,
    onChange: (slot_bar_index:number, slot_index:number, slot:SlotModel) => void,
}

const SlotBar = ({ className, slots, onChange }: Props) => {
    const { isShowing, toggle } = useModal();
    const [currentSlotId, setCurrentSlotId] = useState(-1)
    const [currentBarIndex, setCurrentBarIndex] = useState(0)
    const toogleSlot = (id: number) => {
        setCurrentSlotId(id)
        toggle()
    }

    return (
        <>
            <SlotModal isShowing={isShowing} hide={toggle} index={currentSlotId} slot={slots[currentBarIndex].slots[currentSlotId]} onChange={onChange} barIndex={currentBarIndex} indexName={currentSlotId +""}/>
            <div className={className}>

                <div className="slots">
                    {slots[currentBarIndex].slots.map((slot, index) =>  (
                        <Slot key={index} type={slot.slot_type} index={index} toggleSlotModal={() => toogleSlot(index)} indexName={index +""} />

                    ))}
                    <div className="slotIndexChanger">
                        <button className={`changeBtn ${currentBarIndex === 8 ? 'grayedOutBtn' : ''}`} onClick={() => {
                            if (currentBarIndex != 8) {
                                setCurrentBarIndex(currentBarIndex+1)
                            }
                        }}>
                            &#9650;
                        </button>
                        <button className={`changeBtn ${currentBarIndex === 0 ? 'grayedOutBtn' : ''}`} onClick={() => {
                            if (currentBarIndex != 0) {
                                setCurrentBarIndex(currentBarIndex-1)
                            }
                        }}>
                            &#x25BC;
                        </button>
                    </div>
                    <div className="slotIndexDisplay">
                        {currentBarIndex + 1}
                    </div>
                </div>
            </div>
        </>
    )
}

export default styled(SlotBar)`

    & .slots {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        gap: .25rem;
        background: hsla(0,0%,0%,.75);
        backdrop-filter: blur(.5rem);
        padding: .5rem 2.5rem 1rem 2.5rem;
        border-radius: 100rem;
        border: 1px solid hsl(48,58%,43%);
    }

    & .slotIndexChanger {
        position: absolute;
        top: 10px;
        left: 10px;
        display: flex;
        flex-direction: column;

    }

    & .slotIndexDisplay {
        position: absolute;
        top: 20px;
        right: 20px;
        color: rgb(240, 209, 105);
        font-size: 1.2rem;


    }
    & .grayedOutBtn {
        background-color: rgb(140, 140, 140) !important;
    }
    & .changeBtn {
        border-radius: 50%;
        height: 25px;
        width: 25px;
        text-align: center;
        font-size: 10px;
        background-color: rgb(240, 209, 105);

    }
    & .changeBtn:hover {
        background-color: rgb(255, 236, 154);
    }

`
