import styled from 'styled-components'
import { WebviewWindow } from '@tauri-apps/api/window'

import FlyffLogo from './assets/logo.png'
import LauncherBackground from './assets/launcher_background_ice.png'
import { useState } from 'react'
import BotVisualizer from './BotVisualizer'

type Props = {
    className?: string,
}

const Launcher = ({ className }: Props) => {
    const [isLaunched, setIsLaunched] = useState(false)

    const launch = () => {
        const webview = new WebviewWindow(`client`, {
            title: 'Flyff Universe',
            url: 'https://universe.flyff.com/play'
        })
    
        webview.once('tauri://created', function () {
            webview.show()
            setIsLaunched(true)
        })

        webview.once('tauri://error', function (e) {
            console.error(e)
        })
    }

    return (
        <div className={className}>
            {!isLaunched && (
                <div className="container">
                    <img className="logo" alt="Flyff Universe Logo" src={FlyffLogo} />
                    <div className="btn" onClick={launch}>Play</div>
                </div>
            )}
            {isLaunched && (
                <BotVisualizer />
            )}
        </div>
    )
}

export default styled(Launcher)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100vw;
    height: 100vh;
    background-image: url(${LauncherBackground});
    background-attachment: fixed;
    background-repeat: no-repeat;
    background-position: center center;
    background-size: cover;

    & .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: calc(min(25vh, 20rem));
        max-height: 100vh;
    }

    & .logo {
        height: 250px;
        object-fit: scale-down;
    }

    & .btn {
        cursor: pointer;
        user-select: none;
        padding: 1rem 1rem;
        width: calc(min(500px, max(250px, 40vw)));
        text-align: center;
        border-radius: 0.25rem;
        color: white;
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        transition: all .1s linear;
        box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1);
        border: 1px solid hsl(0,0%,10%);
        font-size: 1.5rem;

        &:hover {
            background: hsla(203, 100%, 45%, .5);
            border: 1px solid hsl(0,0%,10%);
            box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1), 0 .5rem 2rem 0 hsla(0,0%,0%,.25), 0 2rem 2rem 0 hsla(0,0%,0%,.25);
        }
    }
`