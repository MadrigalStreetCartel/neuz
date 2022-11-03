import React, { useEffect, useRef, useState } from 'react'
import styled from 'styled-components'
import { BsDiscord, BsGithub } from 'react-icons/bs'

import SocialButton from './SocialButton'
import useModal from './utils/UseModal'
import YesNoModal from './YesNoModal'
import { getVersion } from '@tauri-apps/api/app'

type Props = {
    className?: string,
    lastVersion?: string,
    currentVersion?: string,
}

const Footer = ({ className, lastVersion: newVersion = "NaN", currentVersion = "NaN" }: Props) => {

    return (
        <>
            <footer className={className}>
                <SocialButton icon={BsDiscord} label="Join our Discord" href="https://discord.gg/cZr3X3mCnq" />
                {currentVersion !== "NaN" && (<p id="versionNumber">V{currentVersion}
                    {(newVersion !== "NaN" && newVersion !== currentVersion) && (<a target="_blank" href="https://github.com/MadrigalStreetCartel/neuz" className="badge">NEW UPDATE</a>)}</p>)
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
