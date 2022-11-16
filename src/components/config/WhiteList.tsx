import styled from "styled-components"
import { useRef, useState } from 'react';
import ConfigTable from "./ConfigTable";

type Props = {
    className?: string,
    whitelist: [number, number, string][],
    onChange: (value: [number, number, string][]) => void,
}

const WhiteList = ({ className, whitelist, onChange }: Props) => {
    const [whiteListValues, setWhiteListValues] = useState(whitelist ?? []);
    //const removeID = useRef<number | null>(null)
    const [removeID, setRemoveID] = useState<number | null>(null)
    return (
        <div className={className}>
            <table>
                <tr>
                    <th>Width</th>
                    <th>Height</th>
                    <th>Name (optionnal)</th>
                </tr>
                {whitelist.map((val, key) => (
                    <tr key={key} className={`${removeID === key? "selected" : ""}`} onClick={() => setRemoveID(key)}>
                        <td>{val[0]}</td>
                        <td>{val[1]}</td>
                        <td>{val[2]}</td>
                    </tr>
                ))}
            </table>
            <div className="btn" onClick={() => onChange?.(whitelist.filter((value, key)=> key !== removeID  ))}>Remove</div>
        </div>
    )
}

export default styled(WhiteList)`
    display: flex;
    flex-direction: column;
    justify-content: center;
    background: hsla(0,0%,0%,.75);
    width: 100%;
    flex-grow: 1;
    border-radius: .25rem;
    position: relative;
    border: 2px solid hsla(0,0%,0%,.75);
    padding: .1rem .25rem;


    & textarea {
        all: unset;
        margin-bottom: .5rem;
    }

    & .btn {
        width: 100% !important;
        font-size: 1rem !important;
        height: 2rem !important;
        padding: .25rem 1rem !important;
    }

    & table {
        font-family: arial, sans-serif;
        border-collapse: collapse;
        width: 100%;
        display: inline-block;
        height: 10em;
        overflow-y: scroll;
    }

    & td, th {
        text-align: left;
        padding: 8px;
    }

    & td:last-child, th:last-child {
        width: 100%
    }

    & .selected {
        background-color: #242424 !important;
    }
`
