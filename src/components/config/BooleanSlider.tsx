import styled from "styled-components"

type Props = {
    className?: string,
    value: boolean,
    onChange: (value: boolean) => void,
}

const BooleanSlider = ({ className, value, onChange }: Props) => {
    return (
        <div className={className} onClick={() => onChange(!value)}>
            <div className="symbol" data-active={value} />
        </div>
    )
}

export default styled(BooleanSlider)`
    display: flex;
    align-items: center;
    background: hsla(0,0%,0%,.75);
    width: 2.5rem;
    height: 1rem;
    border-radius: 10rem;
    position: relative;
    border: 2px solid hsla(0,0%,0%,.75);
    cursor: pointer;

    & .symbol {
        left: -2px;
        position: absolute;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 1.25rem;
        height: 1.25rem;
        border-radius: 10rem;
        transition: all .1s ease-in-out;
        border: 2px solid hsla(0,0%,100%,.05);
        box-shadow: 0 .1rem .1rem hsla(0,0%,0%,1);
        z-index: 1;

        &[data-active=false] {
            margin-left: 0rem;
            background: hsla(0,75%,45%,.75);
        }

        &[data-active=true] {
            margin-left: calc(2.5rem - 1.25rem + 4px);
            background: hsla(121,58%,55%,.75);
        }
    }

    &:hover .symbol {
        border: 2px solid hsla(0,0%,100%,.1);
        box-shadow: 0 .1rem .1rem hsla(0,0%,0%,1), 0 .2rem .33rem hsla(0,0%,0%,.25);
    }
`