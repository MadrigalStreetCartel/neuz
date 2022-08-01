import styled from 'styled-components'

import Slot from '../Slot'

import { SlotType, SlotBarModel, SlotModel, SlotTypes } from '../../models/BotConfig'
import ConfigCollapsible from '../config/ConfigCollapsible'
import { useCallback, useEffect, useState } from 'react'

type Props = {
    className?: string,
    slots: SlotBarModel,
    parentSlot:SlotModel,
    parentId:number,
    onChange: (type: SlotModel, index: number) => void,
}
const SlotList = ({ className, slots,parentSlot, parentId,onChange }: Props) => {

    const [currentId, setCurrentId] = useState(parentId)
    const [currentSlot, setCurrentSlot] = useState<SlotModel>(parentSlot)

    const select = (slot:SlotModel,index:number) => {
        setCurrentId(index)
        setCurrentSlot(slot )
    }
    useEffect(()=> {
        setCurrentId(parentId)
        setCurrentSlot(parentSlot )
    }, [parentId,parentSlot])
    useEffect(()=> {
        parentId =currentId
        parentSlot = currentSlot
        onChange(currentSlot,currentId)
    }, [currentId,currentSlot])
    return (
        <>
            <div className={className}>
                {(currentId > -1) &&
                    <div className="slotConfigPanel">
<></>
                            <ConfigCollapsible open={currentId!=-1}>
                                <></>
                                <div className="slotConfigSlot">
                                    {SlotTypes.map((slotType:SlotType,index:number) => (
                                        <Slot hideIndex isSelected={currentSlot.slot_type == slotType  } onSelected={()=>select({
                                            slot_type: slotType,
                                            slot_cooldown: 0,
                                        },parentId)  } key={index} type={slotType} index={index} />
                                    ))}
                                </div>
                            </ConfigCollapsible>

                    </div>
                }


            </div>


        </>
    )
}

export default styled(SlotList)`
background: none !important;
backdrop-filter: none !important;


    & .slotBarList {
        background: none !important;
        backdrop-filter: none !important;

    }
    & .slotConfigPanel {
        background: none !important;
        backdrop-filter: none !important;
        padding-top:0px !important;

    }
    & .slotConfigSlot {
        color: hsla(0,0%,0%,.75);
        display: flex;
        flex-direction: row;
        justify-content: center;
        gap: .30rem;
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
