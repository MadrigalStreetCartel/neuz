import styled from 'styled-components'

type Props = React.PropsWithChildren<{
    className?: string,
}>

const ConfigPanel = ({ className, children }: Props) => {
    return (
        <div className={className}>
            {children}
        </div>
    )
}

export default styled(ConfigPanel)`
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
    max-width: calc(min(500px, 90vw));
`