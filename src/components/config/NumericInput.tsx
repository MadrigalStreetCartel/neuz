import styled from "styled-components"

type Props = {
    className?: string,
    value: number | false,
    onChange: (value: number) => void,
}

const StringInput = ({ className, value, onChange }: Props) => {
    return (
            <input style={{color:"white"}} className={className} value={value.toString()} onChange={(e)=>onChange(parseInt(e.target.value))} type="number" />            
    )
}

export default styled(StringInput)`
    display: flex;
    align-items: center;
    background: hsla(0,0%,0%,.75);
    width: 25rem;
    height: 1rem;
    border-radius: 10rem;
    position: relative;
    border: 2px solid hsla(0,0%,0%,.75);
    cursor: pointer;
`