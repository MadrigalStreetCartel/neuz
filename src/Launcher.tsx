import { useReducer, useState, useMemo, useRef, useEffect } from 'react'
import { invoke, process as TauriProcess } from '@tauri-apps/api'
import { sample } from 'lodash'
import { WebviewWindow } from '@tauri-apps/api/window'
import styled from 'styled-components'

import FlyffLogo from './assets/msc_dark.png'
import LauncherBackground from './assets/launcher_background_ice.png'
import LauncherBackground2 from './assets/launcher_background.png'
import MissionControl from './MissionControl'
import { randomNumberInRange } from './components/utils/RandomInt'
import useModal from './components/utils/UseModal'
import { getVersion } from '@tauri-apps/api/app'
import YesNoModal from './components/YesNoModal'
import { profile } from 'console'
import { emit } from '@tauri-apps/api/event'
import NumericInput from './components/config/NumericInput'
import ConfigLabel from './components/config/ConfigLabel'
import ConfigTableRow from './components/config/ConfigTableRow'
import ConfigPanel from './components/config/ConfigPanel'
import ConfigTable from './components/config/ConfigTable'
import ProfileDisplay from './components/ProfileDisplay'
import Select from 'react-select'
import TextInput from './components/config/TextInput'

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
        if (!hasEnteredMainLoop) {
            enterMainLoop()
            invoke('create_window',{profileId: profileId}).then(()=> {
                invoke('start_bot',{profileId: profileId}).then(()=> {setIsLaunched(true)})
            })

        }
    }

    const [currentVersion, setCurrentVersion] = useState("NaN")
    const lastVersion = useRef("NaN")
    const updateModal = useModal()
    const newProfileModal = useModal()
    const renProfileModal = useModal()
    const copyProfileModal = useModal()


    const delProfileModal = useModal()

    const [profileId, setPID] = useState<string | null>(null)
    const [idList, setList] = useState<string[]>(["DEFAULT"])
    const [currentPage,setPage] = useState(1)
    const [newProfile,setNewProfile] = useState("")

    const getData=()=>{
        fetch('https://raw.githubusercontent.com/MadrigalStreetCartel/neuz/main/updater.json')
        .then(function(response){
            return response.json();
        })
        .then(function(myJson) {
            if(currentVersion !== "NaN" && myJson.version !== currentVersion) {
                lastVersion.current = myJson.version
                updateModal.open()
            }
        });
    }
    if (currentVersion === "NaN") {
        getVersion().then((value) => {
            setCurrentVersion(value)
        })

    }
    useEffect(() => {
        getData()
        invoke('get_profiles').then((value: any)=> {
            if(value) setList(value);
        })
    },[currentVersion])

    return (
            <div className={className}>
                <YesNoModal isShowing={updateModal.isShown} hide={updateModal.close}
                    title={<h4>Update available!</h4>}
                    body={
                        <>
                            <p>Version V{lastVersion.current} is available! </p>
                            <p>You're currently using V{currentVersion}.</p>
                            <p>Do you want to open download page ?</p>
                        </>
                    }
                    onYes={() => {window.open("https://github.com/MadrigalStreetCartel/neuz")}}
                />
                <YesNoModal isShowing={newProfileModal.isShown} hide={newProfileModal.close}
                    title={<h4>New profile</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value) }} />
                    }
                    onYes={() => {
                        if (!idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('create_profile', {profileId:newProfile.toUpperCase()})
                            setList((oldValue) => [...oldValue, "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />
                <YesNoModal isShowing={renProfileModal.isShown} hide={renProfileModal.close}
                    title={<h4>Rename profile {profileId?.replaceAll("profile_", "")}</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value) }} />
                    }
                    onYes={() => {
                        if (!idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('rename_profile', {profileId: profileId, newProfileId: newProfile})
                            setList((oldValue) => [...oldValue.filter((val) => val.replaceAll("profile_", "") != profileId), "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />

                <YesNoModal isShowing={copyProfileModal.isShown} hide={copyProfileModal.close}
                    title={<h4>Profile {profileId?.replaceAll("profile_", "")} will be copied</h4>}
                    body={
                        <TextInput unit='#' value={newProfile} onChange={(value) => {setNewProfile(value) }} />
                    }
                    onYes={() => {
                        if (!idList.includes("profile_" + newProfile.toUpperCase())){
                            invoke('copy_profile', {profileId: profileId, newProfileId: newProfile})
                            setList((oldValue) => [...oldValue, "profile_" + newProfile.toUpperCase()])
                        }
                        setNewProfile("")
                    }}
                />
                <YesNoModal isShowing={delProfileModal.isShown} hide={delProfileModal.close}
                    title={<h4>Do you want to delete this profile</h4>}
                    body={
                       <h3>This action cant be undone</h3>
                    }
                    onYes={() => {
                        invoke('remove_profile', {profileId: profileId})
                        setList((oldValue) => oldValue.filter((filtred) => filtred.replaceAll("profile_","") != profileId))
                        setPage(1)
                        setPID(null)
                    }}
                />
                {!isLaunched && (
                    <div className="container">
                        <div className="logo-container">
                            <img className="logo" alt="Flyff Universe Logo" src={FlyffLogo} />
                            <span className="greet">{greeting}</span>
                        </div>
                        <ConfigPanel>
                            <ConfigTable>
                                <ConfigTableRow
                                    layout="v"
                                    label={<ConfigLabel name={"Profiles | Current: " + profileId} helpText={"Select/create/edit/copy you're profiles."} />}
                                    item={
                                        <>
                                            <div>
                                                <div className="btn m" onClick={newProfileModal.open}>New</div>
                                                <div className="btn m" onClick={()=> {profileId != null && renProfileModal.open()}}>Rename</div>
                                                <div className="btn m" onClick={()=> {profileId != null && copyProfileModal.open()}}>Copy</div>
                                                <div className="btn m" onClick={()=> {profileId != null && delProfileModal.open()}}>Remove</div>

                                            </div>

                                            <table id="profiles">
                                                {idList.sort((a,b) => (a > b) ? 1 : ((b > a) ? -1 : 0)).slice( (currentPage -1) * 4,  (currentPage -1) * 4 + 4).map((pid, index) => <>
                                                    {(index %5 != 0 || true) &&<tr className={pid.replaceAll("profile_","") == profileId ? "selected" : ""} onClick={() => setPID(pid.replaceAll("profile_",""))}>
                                                        <td>{(pid.startsWith("profile_")? pid.replace("profile_","") : pid)}</td>
                                                    </tr>}
                                                </>)}
                                                <br />
                                                <div style={{display:"flex", textAlign: "center", alignItems: "center",fontSize: "1rem" }}>
                                                    <div  style={{width: "50%", fontSize: "1.5rem"}} className="btn sm" onClick={()=> {setPage((current) => current == 1? current: current -1)}}>{"<-"}</div>
                                                    Page: {currentPage}/{Math.ceil(idList.length / 4)}
                                                    <div  style={{width: "50%", fontSize: "1.5rem"}} className="btn sm" onClick={()=> {setPage((current) => current == Math.ceil(idList.length / 4)? current: current +1)}}>{"->"}</div>
                                                </div>


                                            </table>

                                        </>
                                    }
                                />
                            </ConfigTable>
                        </ConfigPanel>

                        <div className="btn" onClick={launch}>Play</div>
                    </div>
                )}
                {isLaunched && (
                    <>
                        <ProfileDisplay profileId={profileId?? ""} />
                        <MissionControl currentVersion={currentVersion} lastVersion={lastVersion.current} />
                    </>
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

    & .sm {
        padding: 0.25rem 0.25rem;
        align-self: center;
        width: 60%;
        font-size: 1.25rem;
    }

    & .m {
        padding: 0.5rem 0.5rem;
        align-self: center;
        width: 100%;
        font-size: 1rem;
    }

    & #profiles {
        margin-left: 20px;
        font-family: Arial, Helvetica, sans-serif;
        border-collapse: collapse;
        width: 100%;
    }

    & #profiles td, #profiles th {
        padding: 15px;
    }

    & #profiles tr {
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        text-align: center;
        border-bottom: 1px solid;
    }
    & #profiles tr:nth-child(even){
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
    }

    & #profiles tr:hover {
        background: hsl(0deg 0% 30% / 75%) !important;
        color: black;
    }

    & #profiles .selected {
        background: hsl(0deg 0% 89% / 75%) !important;
        color: black;
    }

    & #profiles th {
        padding-top: 12px;
        padding-bottom: 12px;
        text-align: left;
        background-color: #04AA6D;
        color: white;
    }
`
