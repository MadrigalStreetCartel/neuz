import styled from 'styled-components'
import ConfigLabel from './config/ConfigLabel'
import ConfigTableRow from './config/ConfigTableRow'
import Modal from './Modal'
import ConfigTable from './config/ConfigTable'
import { ReactElement } from 'react'
import { message } from '@tauri-apps/api/dialog'

type Props = {
    className?: string,
    isShowing: boolean,
    title? : ReactElement,
    hide: () => void,
    onYes: () => void,
    onNo?: () => void,
}

const YesNoModal = ({className, isShowing, hide, onYes, onNo, title}: Props) => {
    const onClick = (kind: number) => {
        if(kind === 0) {
            onYes()
        } else if (kind == 1 && onNo) {
            onNo()
        }
        hide()
    }
    return(
        <Modal isShowing={isShowing} onHide={() => onClick(1)} hide={hide} title={title ?? <h4>Confirm</h4>} body={

            <ConfigTable>
                <div style={{display: "inline-block", marginLeft: "50%", transform: "translateX(-50%)"}}>
                    <button onClick={() => onClick(0)}>Yes</button>
                    <button style={{width:"25px", backgroundColor: "transparent"}}></button>
                    <button onClick={() => onClick(1)}>No</button>
                </div>
            </ConfigTable>
        }/>
    )
}


export default styled(YesNoModal)`
`


