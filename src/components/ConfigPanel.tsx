import { useState } from 'react'
import styled from 'styled-components'

import { BotConfigModel } from '../BotVisualizer'
import BooleanSlider from './BooleanSlider'

type ConfigLabelProps = {
    className?: string,
    name: string,
    helpText?: string,
}

const ConfigLabel = ( { className, name, helpText }: ConfigLabelProps) => {
    return (
        <div className={className}>
            {name}
            {helpText && (
                <div className="help">
                    <abbr title={helpText} className="help__symbol">?</abbr>
                </div>
            )}
        </div>
    )
}

const StyledConfigLabel = styled(ConfigLabel)`
    display: flex;
    align-items: center;
    gap: .5rem;

    & .help {
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 50% 50%;
        background-color: hsla(0,0%,0%,.75);
        width: 16px;
        height: 16px;
        border: 1px solid hsl(48,50%,43%);
        cursor: help;
        overflow: hidden;

        &__symbol {
            font-size: .75rem;
            font-weight: bold;
            text-decoration: none;
        }
    }
`

type Props = {
    className?: string,
    config: BotConfigModel,
    onChange?: (config: BotConfigModel) => void,
}

const ConfigPanel = ({ className, config, onChange }: Props) => {
    return (
        <div className={className}>
            <div className="row">
                <StyledConfigLabel name="On-Demand Pickup Pet" helpText="Manages pickup pet activation automatically." />
                <BooleanSlider value={config.on_demand_pet} onChange={value => onChange?.({ ...config, on_demand_pet: value })} />
            </div>
            <div className="row">
                <StyledConfigLabel name="Use skills to attack" helpText="Enables the use of skills configured in the action slot to attack." />
                <BooleanSlider value={config.use_attack_skills} onChange={value => onChange?.({ ...config, use_attack_skills: value })} />
            </div>
        </div>
    )
}

export default styled(ConfigPanel)`
    display: flex;
    flex-direction: column;
    gap: .25rem;
    background: hsla(0,0%,0%,.75);
    backdrop-filter: blur(.5rem);
    padding: 1rem;
    border-radius: .25rem;
    color: white;

    & .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 5rem;
    }
`