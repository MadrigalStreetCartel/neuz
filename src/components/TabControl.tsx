import React from 'react'
import styled from 'styled-components'

import { ModeModel } from '../models/BotConfig'

type Props = React.PropsWithChildren<{
    className?: string,
    activeMode?: ModeModel,
    onSelect: (mode: ModeModel) => void,
}>

const TabControl = ({ className, children, activeMode, onSelect }: Props) => {
    return (
        <div className={className}>
            {React.Children.map(children, child => (
                React.cloneElement(child as React.ReactElement, { activeMode, onSelect })
            ))}
        </div>
    )
}

export default styled(TabControl)`
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0;
`