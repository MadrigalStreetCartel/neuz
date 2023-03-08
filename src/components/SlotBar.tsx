import styled from 'styled-components'

import Slot from './Slot'

import { createSlotBars, FarmingConfigModel, SlotModel, SupportConfigModel } from '../models/BotConfig'
import SlotModal from './SlotModal'
import useModal from './utils/UseModal'
import { useState } from 'react'
import { useKeyPress } from './utils/KeyboardHotkeys'

type Props = {
    className?: string,
    config: FarmingConfigModel | SupportConfigModel,
    onChange: (config: SupportConfigModel | FarmingConfigModel) => void,
    botMode: string
}

const SlotBar = ({ className, config, botMode, onChange }: Props) => {
    const { isShown, toggle } = useModal();
    const [currentSlotId, setCurrentSlotId] = useState(-1)
    const [currentBarIndex, setCurrentBarIndex] = useState(0)
    const toogleSlot = (id: number, event: any,targetSlot: SlotModel ) => {
        if(event.shiftKey){
            targetSlot.slot_enabled = !targetSlot.slot_enabled;
            handleSlotChange(currentBarIndex, id, targetSlot);
        }else{
            setCurrentSlotId(id)
            toggle()
        }
    }
    let slots = config.slot_bars ?? createSlotBars()

    // Bind Neuz slotbars with F1-F9 just like in game
    useKeyPress(["F1","F2","F3","F4","F5","F6","F7","F8","F9",], (event: { key: string }) => {
        setCurrentBarIndex(parseInt(event.key.replace("F","")) - 1)
    })

    const handleSlotChange = (slot_bar_index:number, slot_index:number, slot: SlotModel) => {
        const newConfig = { ...config, slot_bars: config.slot_bars ?? createSlotBars() }
        newConfig.slot_bars[slot_bar_index].slots[slot_index] = slot
        onChange(newConfig)
    }

    return (
        <>
            <SlotModal botMode={botMode} isShowing={isShown} hide={toggle} index={currentSlotId} slot={slots[currentBarIndex].slots[currentSlotId]} onChange={handleSlotChange} barIndex={currentBarIndex} indexName={currentSlotId +""}/>
            <div className={className}>

                <div className="slots">
                    {slots[currentBarIndex].slots.map((slot, index) =>  (
                        <Slot key={index} type={slot.slot_type} index={index} toggleSlotModal={(event,targetSlot) => toogleSlot(index,event,targetSlot)} indexName={index +""} slot={slots[currentBarIndex].slots[index]} />

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
