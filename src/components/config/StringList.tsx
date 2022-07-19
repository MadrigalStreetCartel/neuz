import styled from "styled-components"
import { useState } from 'react';

type Props = {
    className?: string,
    messages?: string[],
    onChange: (value: string[]) => void,
}

const StringList = ({ className, messages, onChange }: Props) => {
    const [messageBuffer, setMessageBuffer] = useState<string[]>(messages ?? []);
    const getValue = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        return e.target.value.trim().length === 0
            ? []
            : e.target.value.split('\n').map(line => line.trim())
    }
    return (
        <div className={className}>
            <textarea value={messageBuffer?.join("\n") ?? ""} onChange={e => setMessageBuffer(getValue(e))} />
            <div className="btn" onClick={() => onChange?.(messageBuffer)}>Apply</div>
        </div>
    )
}

export default styled(StringList)`
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
`
