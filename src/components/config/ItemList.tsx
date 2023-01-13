import styled from "styled-components"
import { useRef, useState } from 'react';
import ConfigTable from "./ConfigTable";

type Props = {
    className?: string,
    items: (any)[][],
    headers: string[],
    onChange: (value: (any)[][]) => void,
    onAdd?: () => void,
    onDoubleClick?: (value: (any)[][]) => void,
    //canAdd?: boolean,
    canRemove?: boolean,
    onBack?: () => void,
}

const ItemList = ({ className, items, headers, onChange, onAdd, onDoubleClick: onSelected, onBack, canRemove = true }: Props) => {
    const [removeID, setRemoveID] = useState<number | null>(null)
    return (
        <div className={className}>
            {onBack && <button className="btn backBtn" onClick={onBack}>Back</button>}
            {onAdd && <button className="btn m" onClick={onAdd}>Add</button>}
            <table>
                <tr>
                {headers.map((val, key) => (<th key={key}>{val}</th>))}
                </tr>
                {items.map((val, itemKey, array) => (
                    <tr onDoubleClick={() => onSelected && onSelected(val) } key={itemKey} className={`${removeID === itemKey? "selected" : ""}`} onClick={() => setRemoveID(itemKey)}>
                        {headers.map(
                            (_, key) => {
                                if (typeof val[key] === "function") return (<td key={key}><button onClick={val[key]}>Click</button></td>)
                                if (typeof val[key] !== "boolean") return (<td key={key}>{val[key]}</td>)
                                if (typeof val[key] === "boolean") return (<td key={key}><input type="checkbox" checked={val[key]} onChange={(value) => {array[itemKey][key] = value.target.checked; onChange(array)}}/></td>)
                            })
                        }
                    </tr>
                ))}
            </table>
            {canRemove && <div className="btn" onClick={() => onChange?.(items.filter((_value, key)=> key !== removeID  ))}>Remove</div>}
        </div>
    )
}

export default styled(ItemList)`
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
