import styled from "styled-components"

type Props = {
    className?: string,
    value: string,
    onChange: (value: string) => void,
}

const StringInput = ({ className, value, onChange }: Props) => {
    return (
            <textarea rows={4} cols={50} style={{color:"white"}} className={className} value={value} onChange={(e)=>onChange(e.target.value)} >
            
            
            
            </textarea>
    )
}

export default styled(StringInput)`
    display: flex;
    align-items: center;
    background: hsla(0,0%,0%,.75);
    width: 25rem;
    height: 1rem;
    position: relative;
    border: 2px solid hsla(0,0%,0%,.75);
    cursor: pointer;

`