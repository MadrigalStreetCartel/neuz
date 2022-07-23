import React from 'react'
import styled from 'styled-components'
import { BsDiscord, BsGithub } from 'react-icons/bs'

import SocialButton from './SocialButton'

type Props = {
    className?: string,
}

const Footer = ({ className }: Props) => {
    return (
        <footer className={className}>
            <SocialButton icon={BsDiscord} label="Join our Discord" href="https://discord.gg/cZr3X3mCnq" />
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
`
