import styled from 'styled-components'

import { ModeModel } from '../models/BotConfig'

type Props = {
    className?: string,
    image: string,
    mode: ModeModel,
    activeMode?: ModeModel,
    onSelect?: (mode: ModeModel) => void,
}

const Tab = ({ className, image, mode, activeMode, onSelect }: Props) => {
    return (
        <div className={[className, activeMode === mode ? `${className}--active` : ''].join(' ').trim()} onClick={() => onSelect?.(mode)}>
            <img src={image} alt="" />
        </div>
    )
}

export default styled(Tab)`
    cursor: pointer;
    border-radius: .25rem;
    overflow: hidden;
    width: 150px;
    transition: all .25s ease;
    filter: grayscale(100%);
    opacity: 0.9;
    
    &--active {
        filter: grayscale(0%);
        
        img {
            border: 2px solid white;
            border-radius: .25rem;
        }
    }
    
    &:hover {
        transform: scale(1.05);
        filter: grayscale(0%);
    }
    
    & img {
        width: 100%;
        object-fit: scale-down;
        border-radius: .25rem;
        backdrop-filter: blur(.5rem);
    }
`