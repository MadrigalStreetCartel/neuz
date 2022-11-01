import styled from "styled-components"

type Props = {
    className?: string,
    value: number | undefined,
    unit?: string,
    onChange: (value: any) => void,
    max?: number;
    min?: number;
}

const NumericInput = ({ className, value, unit, min, max, onChange }: Props) => {
    return (
        <div className={className}>
            <input min={min} max={max} type="number" value={value?.toString() ?? ""} onChange={(e)=>onChange(e.target.value ? parseInt(e.target.value) : null)} />
            {unit && <span className="unit">{unit}</span>}
        </div>
    )
}

export default styled(NumericInput)`
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
