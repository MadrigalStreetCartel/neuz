import styled from "styled-components"

type Props = {
    className?: string,
    value: string | undefined,
    unit?: string,
    min?: number,
    max?: number,
    onChange: (value: string) => void,
    condition?: (value: any) => boolean,
}

const TextInput = ({ className, value, unit,min = 0, max, onChange, condition }: Props) => {
    return (
        <div className={className}>
            <input type="text" value={value?.toString() ?? ""} onChange={(e)=>onChange(e.target.value)} />
            {unit && <span className="unit">{unit}</span>}
        </div>
    )
}

export default styled(TextInput)`
    display: flex;
    align-items: center;
    background: hsla(0,0%,0%,.75);
    width: 100%;
    flex-grow: 1;
    border-radius: .25rem;
    position: relative;
    border: 2px solid hsla(0,0%,0%,.75);
    padding: .1rem .25rem;
    color: white;

    & input {
        all: unset;
        display: flex;
        flex-grow: 1;
        height: 100%;
    }

    & .unit {
        margin-left: .5rem;
    }
`
