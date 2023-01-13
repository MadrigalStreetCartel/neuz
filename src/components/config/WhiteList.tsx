import styled from "styled-components"
import { useRef, useState } from 'react';
import ConfigTable from "./ConfigTable";
import ItemList from "./ItemList";

type Props = {
    className?: string,
    whitelist: (any)[][],
    onChange: (value: (any)[][]) => void,
    onAdd: () => void,
}

const WhiteList = ({ className, whitelist, onChange, onAdd }: Props) => {
    return (
       <ItemList items={whitelist} onChange={onChange} onAdd={onAdd} headers={["Width", "Height", "Name (optionnal)"]} />
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
