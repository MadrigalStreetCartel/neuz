import { useState } from "react";
import styled from "styled-components"
import { StopWatchValues } from "../utils/StopWatch";

type Props = {
    className?: string,
    value: number | undefined,
    onChange: (value: any) => void,

}

const TimeInput = ({ className, value, onChange }: Props) => {
    let stopWatch = new StopWatchValues(value ?? 0)
    const [newValue, setNewValue] = useState(stopWatch.toString())

    return (
        <div className={className}>
            <input type="text" value={newValue?.toString() ?? ""}
                onChange={(e)=>{
                    stopWatch.fromString(e.target.value)
                        if (stopWatch.timer > 0 || e.target.value == stopWatch.toString() ) {
                        onChange(stopWatch.timer)
                    }
                    setNewValue(e.target.value)
                }}
            />
            {!stopWatch.validStringFormat(newValue) ?
                <span className="unit" style={{color:"red"}} onClick={() => {setNewValue(stopWatch.toString())}}>Invalid</span> : <span className="unit" style={{color:"green"}}>Valid</span>
            }
        </div>
    )
}

export default styled(TimeInput)`
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
