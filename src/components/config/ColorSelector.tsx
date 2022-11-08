import styled from "styled-components"

type Props = {
    className?: string,
    value: number[],
    onChange: (value: number[]) => void,
}

const NumericInput = ({ className, value,  onChange }: Props) => {
    return (
        <div className={className}>
            <span className="unit">R </span>
            <input min={0} max={255} type="number" value={value[0]} onChange={(e)=>{value[0] = e.target.valueAsNumber;onChange(value)}} />
            <span className="unit">G </span>
            <input min={0} max={255} type="number" value={value[1]} onChange={(e)=>{value[1] = e.target.valueAsNumber;onChange(value)}} />
            <span className="unit">B </span>
            <input min={0} max={255} type="number" value={value[2]} onChange={(e)=>{value[2] = e.target.valueAsNumber;onChange(value)}} />
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
