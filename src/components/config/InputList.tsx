import styled from "styled-components"
import { useState } from 'react';
import ConfigRow from "./ConfigRow";

type Props = {
    className?: string,
    value: string[],
    onChange: (value: string[]) => void,
}

const StringInput = ({ className, value, onChange }: Props) => {
    let word = "";
    const [dummy,setDummy] = useState(true);
    const [words,setWords] = useState<string[]>(value);
    return (
        <>
        
            {words.map((word, i) => 
                <span style={{border:"2px solid hsla(0,0%,0%,.75)"}}>{i}# {word} <span style={{float:"right",border:"1px solid"}} onClick={() => {setWords(words.filter(x=> x !== word));onChange(words)}}>X</span></span>
                )           
            }
           
            <ConfigRow>
            <textarea rows={10} cols={200} style={{color:"white", height:"25px",maxHeight:"400px"}} className={className} onChange={(e)=>{
                word = e.target.value
            }} >
            
            </textarea>
            <button onClick={()=>{
                if(word.length>0){
                    words.push(word)
                    setWords(words)
                    setDummy(!dummy)
                    onChange(words)
                }

                
            }}>
                Add
            </button>
            </ConfigRow>
        </>
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