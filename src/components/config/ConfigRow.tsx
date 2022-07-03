import styled from 'styled-components'

type Props = {
    className?: string,
    children: React.ReactNode,
}

const ConfigRow = ({ className, children }: Props) => {
    return (
        <div className={className}>
            {children}
        </div>
    )
}

export default styled(ConfigRow)`
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 1rem;
`