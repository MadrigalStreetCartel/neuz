import React, { ReactElement } from 'react'
import styled from 'styled-components'


type Props = {
    className?: string,
    isShowing: boolean,
    hide: () => void,
    body?: ReactElement;
    title?: ReactElement;
    closeBtn?: boolean;

}

const SlotModal = ({className, isShowing, hide, body, title,closeBtn = true }: Props) => {
    if (isShowing) {
        return(
            <div className={className} >
                <div className="modal-overlay">
                    <div className="modal-wrapper" onMouseDown={event => { if (event.target === event.currentTarget)  hide()}}>
                    <div className="modal">
                        <div className="modal-header">
                            {title &&
                                title
                            }
                            {closeBtn &&
                                <button
                                    type="button"
                                    className="modal-close-button"
                                    onClick={hide}
                                >
                                    <span style={{color:"white"}}>&times;</span>
                                </button>
                            }
                        </div>
                        <div className="modal-body">
                            {body &&
                               body
                            }
                        </div>
                    </div>
                    </div>
                </div>
            </div>
    )
    }else {
        return(<></>)
    }

}


export default styled(SlotModal)`
position: absolute;
& .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 1040;
    background-color: rgba(0, 0, 0, 0.2);
}
&.select-slot {
    color: inherit;
    color: black !important;
    display: inline-block;
    fontSize: 12;
    fontStyle: italic;
    marginTop: 1em;
}
& .modal-wrapper {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 1050;
    width: 100%;
    height: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    outline: 0;
    display: flex;
    align-items: center;
}

& .modal {
    z-index: 100;
    background: #fff;
    position: relative;
    margin: auto;
    border-radius: 5px;
    max-width: 500px;
    height:auto;
    width: 80%;
    padding: 1rem;
    background: hsla(0,0%,0%,.75);
    backdrop-filter: blur(.9rem);
}

& .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: white;

}

& .modal-close-button {
    font-size: 1.4rem;
    font-weight: 700;
    color: #000;
    cursor: pointer;
    border: none;
    background: transparent;
}

&.modal-body {
    display: flex;
    flex-wrap: wrap;
}


& .slot {
    text-align: center;
    background-color: hsla(0,0%,100%,.05);
}

& .desc {
    width: 100%;
    text-align: center;
    font-size: 1rem;
    }

& div.type {
    color: white;
    font-size: 1.5rem;
}

& img.type {
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: .25rem;
    border-radius: .25rem;
}

`


