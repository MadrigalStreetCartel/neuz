import React, { useEffect, useRef, useState } from 'react'
import styled from 'styled-components'
import { BsDiscord, BsGithub } from 'react-icons/bs'

import SocialButton from './SocialButton'
import useModal from './utils/UseModal'
import YesNoModal from './YesNoModal'

type Props = {
    className?: string,
    version?: number[],
}

const Footer = ({ className, version }: Props) => {

    const [currentVersion, setCurrentVersion] = useState("NaN")
    const newVersion = useRef("NaN")
    const updateModal = useModal()

    const getData=()=>{
        fetch('https://raw.githubusercontent.com/MadrigalStreetCartel/neuz/main/updater.json')
        .then(function(response){
            return response.json();
        })
        .then(function(myJson) {
            if(currentVersion !== "NaN" && myJson.version !== currentVersion) {
                newVersion.current = myJson.version
                updateModal.open()
            }
        });
    }
    if (version && currentVersion === "NaN") {
        setCurrentVersion(`${version[0]}.${version[1]}.${version[2]}`)
    }
    useEffect(() => {
        getData()
    },[currentVersion])


    return (
        <>
            <YesNoModal isShowing={updateModal.isShown} hide={updateModal.close}
            title={<h4>Update available!</h4>}
            body={
                <div>
                    <p>Version V{newVersion.current} is available! </p>
                    <p>You're currently using V{currentVersion}.</p>
                    <p>Do you want to open download page ?</p>
                </div>
            }
            onYes={() => {window.open("https://github.com/MadrigalStreetCartel/neuz")}}/>


            <footer className={className}>
                <SocialButton icon={BsDiscord} label="Join our Discord" href="https://discord.gg/cZr3X3mCnq" />
                {version && (<p id="versionNumber">V{currentVersion}
                    {(newVersion.current !== "NaN" && newVersion.current !== currentVersion) && (<a target="_blank" href="https://github.com/MadrigalStreetCartel/neuz" className="badge">NEW UPDATE</a>)}</p>)
                }
                <SocialButton icon={BsGithub} label="Star us on GitHub" href="https://github.com/MadrigalStreetCartel/neuz" />
            </footer>
        </>
    )
}

export default styled(Footer)`
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    border-top: 2px solid hsla(0,0%,0%,.1);
    background: hsla(0,0%,0%,.25);
    backdrop-filter: blur(.25rem);
    margin-top: 1rem;
    gap: 1rem;
    padding: .25rem;

    & #versionNumber {
        color: white;
    }

    & .badge {
        text-decoration:none;
        background-color:#000;
        color: yellow;
        display:inline-block;
        padding-left:8px;
        padding-right:8px;
        text-align:center;
        border-radius:50%;
        margin-top:-15px;
        font-size: 12px;
        margin-left:-70px;
        position: fixed;
    }
`
