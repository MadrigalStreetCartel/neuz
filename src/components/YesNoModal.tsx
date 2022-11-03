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
    title?: ReactElement,
    body?: ReactElement,
    hide: () => void,
    onYes: () => void,
    onNo?: () => void,
}

const YesNoModal = ({className, isShowing, hide, onYes, onNo, title, body}: Props) => {
    const onClick = (kind: number) => {
        if(kind === 0) {
            onYes()
        } else if (kind == 1 && onNo) {
            onNo()
        }
        hide()
    }
    return(
        <div className={className}>
            <Modal isShowing={isShowing} onHide={() => onClick(1)} hide={hide} title={title ?? <h4>Confirm</h4>} body={
                <>
                    {body && <div className='modalBody'> {body} </div>}
                    <div className="buttonHolder">
                        <button className='btn sm' onClick={() => onClick(0)}>Yes</button>
                        <button className='btn sm' onClick={() => onClick(1)}>No</button>
                    </div>
                </>
            }/>
        </div>

    )
}


export default styled(YesNoModal)`
    & .modalBody {
        text-align: center;
    }

    & .buttonHolder {
        display: flex;
        gap: 32px;
        margin-left: 50%;
        transform: translateX(-50%)
    }
`


