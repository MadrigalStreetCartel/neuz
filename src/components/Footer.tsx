import React, { useEffect, useState } from 'react'
import styled from 'styled-components'
import { BsDiscord, BsGithub } from 'react-icons/bs'

import SocialButton from './SocialButton'

type Props = {
    className?: string,
    version?: number[],
}

const Footer = ({ className, version }: Props) => {
    const [newVersion, setNewVersion] = useState("0.0.0")
    const [currentVersion, setCurrentVersion] = useState("NaN")
    const [updateAvailible, setUpdateAvailible] = useState(false)
    const getData=()=>{
        fetch('https://raw.githubusercontent.com/MadrigalStreetCartel/neuz/clean-local-testing/updater.json')
        .then(function(response){
            return response.json();
        })
        .then(function(myJson) {
            if(myJson.version !== "0.0.0") {
                setNewVersion(myJson.version)
                console.log(newVersion)
                if(currentVersion !== "NaN" && newVersion !== currentVersion) {
                    setUpdateAvailible(true)
                }
            }
        });
    }
    if (version && currentVersion === "NaN") {
        setCurrentVersion(`${version[0]}.${version[1]}.${version[2]}`)
    }
    useEffect(()=>{
        getData()
    },[])

    return (
        <footer className={className}>
            <SocialButton icon={BsDiscord} label="Join our Discord" href="https://discord.gg/cZr3X3mCnq" />
            {version && (<p id="versionNumber">V{currentVersion}
                {updateAvailible && (<a target="_blank" href="https://github.com/MadrigalStreetCartel/neuz" className="badge">NEW UPDATE</a>)}</p>)
            }
            <SocialButton icon={BsGithub} label="Star us on GitHub" href="https://github.com/MadrigalStreetCartel/neuz" />
        </footer>
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
        color:#fff;
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
