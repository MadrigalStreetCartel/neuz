import { useRecoilState } from 'recoil'
import styled from 'styled-components'

import { activeHelpState } from '../../state/atoms'

type Props = {
    className?: string,
    name: string,
    helpText?: string,
}

const ConfigLabel = ( { className, name, helpText }: Props) => {
    const [activeHelpId, setActiveHelpId] = useRecoilState(activeHelpState)
    const showHelp = activeHelpId === name

    return (
        <div className={className}>
            <div className="inner">
                <span className="name">{name}</span>
                {helpText && (
                    <div className="help" onClick={() => setActiveHelpId(showHelp ? null : name)}>
                        <abbr title={helpText} className="help__symbol">?</abbr>
                    </div>
                )}
            </div>
            {helpText && showHelp && (
                <div className="help-content">
                    {helpText}
                </div>
            )}
        </div>
    )
}

export default styled(ConfigLabel)`
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: .5rem;
    color: white;

    & .inner {
        display: flex;
        align-items: center;
        gap: .5rem;
    }

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

    & .help-content {
        border-radius: .25rem;
        border-left: 2px solid hsla(0,0%,100%,.75);
        padding: .5rem 1rem .5rem 1rem;
        word-wrap: break-word;
        background: hsla(0,0%,100%,.1);
        font-size: .9rem;
    }
`
