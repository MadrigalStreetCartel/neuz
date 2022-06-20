import styled from 'styled-components'
import { FixedArray, SlotModel, SlotType } from '../BotVisualizer'
import Slot from './Slot'

type Props = {
    className?: string,
    slots: FixedArray<SlotModel, 10>,
    onChange?: (type: SlotType, index: number) => void,
}

const SlotBar = ({ className, slots, onChange }: Props) => {
    return (
        <div className={className}>
            {slots.map((slot, index) => (
                <Slot key={index} type={slot.slot_type} index={index} onChange={type => onChange?.(type, index)} />
            ))}
        </div>
    )
}

export default styled(SlotBar)`
    display: flex;
    align-items: center;
    justify-content: center;
    gap: .25rem;
    background: hsla(0,0%,0%,.75);
    backdrop-filter: blur(.5rem);
    padding: .5rem 2.5rem .75rem 2.5rem;
    border-radius: 100rem;
    border: 1px solid hsl(48,58%,43%);
`