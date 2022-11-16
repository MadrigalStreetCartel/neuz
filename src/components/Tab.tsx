import styled from 'styled-components'

import { ModeModel } from '../models/BotConfig'

type Props = {
    className?: string,
    image?: string,
    text?: string,
    mode: ModeModel,
    activeMode?: ModeModel,
    onSelect?: (mode: ModeModel) => void,
}

const Tab = ({ className, image, text, mode, activeMode, onSelect }: Props) => {
    return (
        <div className={[className, activeMode === mode ? `${className}--active` : ''].join(' ').trim()} onClick={() => onSelect?.(mode)}>
            {image && <img src={image} alt="" />}
            {text && text}
        </div>
    )
}

export default styled(Tab)`
    cursor: pointer;
    border-radius: .25rem;
    overflow: hidden;
    height: 75px;
    transition: all .25s ease;
    filter: grayscale(100%);
    opacity: 0.9;
    background-color: black;
    color: white;
    width: 146px;
    height: 75px;
    line-height: 75px;
    text-align: center;

    &--active {
        filter: grayscale(0%);


        border: 2px solid white;
        border-radius: .25rem;

    }

    &:hover {
        transform: scale(1.05);
        filter: grayscale(0%);
    }

    & img {
        width: 100%;
        height: 75px;
        object-fit: scale-down;
        border-radius: .25rem;
        backdrop-filter: blur(.5rem);
    }
`
