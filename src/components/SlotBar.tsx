import styled from 'styled-components'

import Slot from './Slot'

import { SlotType, SlotBarModel, SlotModel, CooldowwnSlotTypes, FarmingConfigModel } from '../models/BotConfig'
import ConfigCollapsible from './config/ConfigCollapsible'
import { useCallback, useEffect, useState } from 'react'
import SlotList from './config/ConfigSlotList'
import NumericInput from './config/NumericInput'
import ConfigTableRow from './config/ConfigTableRow'
import ConfigLabel from './config/ConfigLabel'

type Props = {
    className?: string,
    slots: SlotBarModel,
    config: FarmingConfigModel,
    onChange: (type: SlotModel, index: number) => void,

}

const SlotBar = ({ className, slots,config, onChange }: Props) => {

    const [currentId, setCurrentId] = useState(-1)
    const [currentSlot, setCurrentSlot] = useState<SlotModel>()

    const select = (slot:SlotModel,index:number) => {
        setCurrentId(index)
        setCurrentSlot(slot)

        onChange(slot,index)
    }

    const SlotParamsChange =(value:number, selector:string) => {
        if(!currentSlot) return
        switch (selector) {
            case 'slot_priority':
                currentSlot.slot_priority = value
              break;
            case 'slot_cooldown':
                currentSlot.slot_cooldown = value
                break;
            default:
              console.log(`Unable to find type ${selector}.`);
          }
          onChange(currentSlot ,currentId)
    }


    return (
        <>
            <div className={className}>
                <div className="slotBar">
                    {slots.map((slot, index) => (
                        <Slot isSelected={currentId == index} onSelected={() => select(slot,index)} key={index} type={slot.slot_type} index={index} onChange={type => select(slot, index)} />
                    ))}
                </div>

                    {(currentSlot != undefined) &&
                        <div className="slotConfigPanel">
                            <ConfigCollapsible open={currentId!=-1}>
                                    <SlotList parentId={currentId} parentSlot={currentSlot} onChange={select}  slots={slots}></SlotList>
                                    <ConfigTableRow
                                            layout="v"
                                            label={<ConfigLabel name="Priority" helpText="Prioritze you skills." />}
                                            item={<NumericInput value={currentSlot.slot_priority} onChange={value => SlotParamsChange(value,"slot_priority")} />} />
                    {CooldowwnSlotTypes.includes(currentSlot.slot_type) &&
                            <ConfigTableRow
                                        layout="v"
                                        label={<ConfigLabel name="Cooldown" helpText="Interval between cast in milliseconds." />}
                                        item={<NumericInput unit="ms" value={currentSlot.slot_cooldown??0} onChange={value => SlotParamsChange(value,"slot_cooldown")} />} />
                    }
                                <button className='btn' onClick={()=> {setCurrentId(-1);setCurrentSlot(undefined)}}>Close</button>

                            </ConfigCollapsible>
                        </div>
                    }



            </div>


        </>
    )
}

export default styled(SlotBar)`

    & .slotBar {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: .25rem;
        background: hsla(0,0%,0%,.75);
        backdrop-filter: blur(.5rem);
        padding: .5rem 2.5rem 1rem 2.5rem;
        border-radius: 100rem;
        border: 1px solid hsl(48,58%,43%);

    }
    & .slotConfigPanel {
        display: flex;
        flex-direction: column;
        gap: .5rem;
        background: hsla(0,0%,0%,.75);
        backdrop-filter: blur(.5rem);
        padding: 1rem;
        border-radius: .25rem;
        color: white;
        overflow: auto;
        width: 100%;
        max-width: calc(min(600px, 90vw));


    }
    & .SlotConfigChange {
        background: none !important;
        backdrop-filter: none !important;

    }

    & .btn {
        width: 100% !important;
        font-size: 1rem !important;
        height: 2rem !important;
        padding: .25rem 1rem !important;
    }

`
