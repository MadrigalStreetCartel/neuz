import { useReducer, useState, useMemo } from 'react'
import { invoke, process as TauriProcess } from '@tauri-apps/api'
import { sample } from 'lodash'
import { WebviewWindow } from '@tauri-apps/api/window'
import styled from 'styled-components'

import FlyffLogo from './assets/msc_dark.png'
import LauncherBackground from './assets/launcher_background_ice.png'
import LauncherBackground2 from './assets/launcher_background.png'
import MissionControl from './MissionControl'
import { randomNumberInRange } from './components/utils/RandomInt'

const launcherBackgrounds = [LauncherBackground, LauncherBackground2]
const Greetings = [
    '😭 Real Market trading is destroying our game 😭',
    'Gala, if you see this, hire me',
    'I am a bot, I am a bot, I am a bot',
    'Gala vs Computer Vision',
    'Not a virus, guaranteed',
    'Neuz.exe',
    'Remember sunkist?',
    'Select individuals only',
    '...you do the math',
    'Vampire Empire',
    'Certified Based',
    'I am not a bot',
    'My french fries are made out of Gpotatoes',
    'Plz don\'t kill me',
    'Plz don\'t ban me',
    'Gib me a star on github',
    'Why p2w when you can bot',
    'Chad botter vs virgin p2w user',
    'Do you even grind bro',
    'Why waste time with farming',
    'Why waste time when you can bot',
    'Bot & Chill',
    'Don\'t like bots? Don\t play the game',
    'Ich chill bei Bangs',
    'Gala sued me for my pserver 😥',
    'Can be bought with 0 BTC 👍',
    'Free is always best',
    '/say me',
    'MPL licensed',
    'Can you buff me pls?',
    'Plz res me',
    'Smoke ants',
    'Madrigal Street Capital',
    'Madrigal Street Cartel',
    'Madrigal Street Casino',
    'Madrigal Street Chinafarming',
    'Madrigal Street Cocaine',
    'Madrigal Skooma Cartel',
    'Do you remember CFlyff',
    'Sly was here',
    '夢想飛飛',
]

type Props = {
    className?: string,
}

const Launcher = ({ className }: Props) => {
    const [hasEnteredMainLoop, enterMainLoop] = useReducer(() => true, false);
    const [isLaunched, setIsLaunched] = useState(false)
    const greeting = useMemo(() => sample(Greetings), []);

    const launch = () => {
        const webview = new WebviewWindow(`client`, {
            title: 'Flyff Universe',
            url: 'https://universe.flyff.com/play',
            center: true,
            resizable: false,
        })

        webview.once('tauri://created', function () {
            webview.show()
            setIsLaunched(true)

            if (!hasEnteredMainLoop) {
                enterMainLoop()
                invoke('start_bot')
            }
        })

        webview.once('tauri://close-requested', function () {
            webview.close()
            setIsLaunched(false)
            TauriProcess.relaunch();
        })
    }

    return (
        <div className={className}>
            {!isLaunched && (
                <div className="container">
                    <div className="logo-container">
                        <img className="logo" alt="Flyff Universe Logo" src={FlyffLogo} />
                        <span className="greet">{greeting}</span>
                    </div>
                    <div className="btn" onClick={launch}>Play</div>
                </div>
            )}
            {isLaunched && (
                <MissionControl />
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
    background-image: url(${launcherBackgrounds[randomNumberInRange(0,1)]});
    background-attachment: fixed;
    background-repeat: no-repeat;
    background-position: center center;
    background-size: cover;

    & .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: space-around;
        height: 100vh;
    }

    & .logo-container {
        display: flex;
        justify-content: center;
        position: relative;
        width: 100%;
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        padding: 2rem;
        box-shadow: 0 0 .1rem hsla(0,0%,0%,.5), 0 0 .5rem hsla(0,0%,0%,.5);
        border-radius: 0.25rem;

        & .logo {
            width: 100%;
            padding: 0 0 2.5rem 0;
            height: calc(25vh + 2rem);
            max-height: 128px;
            object-fit: scale-down;
            opacity: .9;
        }

        @keyframes wiggle {
            0% {
                transform: scale(1) rotate(0deg);
            }
            33% {
                transform: scale(1.1) rotate(-1deg);
            }
            66% {
                transform: scale(1.1) rotate(1deg);
            }
            100% {
                transform: scale(1) rotate(0deg);
            }
        }

        & .greet {
            position: absolute;
            white-space: nowrap;
            text-align: center;
            font-size: 1.25rem;
            color: white;
            bottom: 1.5rem;
            animation: wiggle 5s ease-in-out infinite;
        }
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
