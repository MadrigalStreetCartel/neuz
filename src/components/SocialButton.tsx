import React from 'react'
import { IconType } from 'react-icons/lib'
import styled from 'styled-components'

type Props = {
    className?: string,
    icon: IconType,
    label: string,
    href: string,
}

const SocialButton = ({ className, icon, label, href }: Props) => {
    const Icon = icon

    return (
        <a className={className} href={href} target="_blank" rel="noreferrer">
            <div className="icon">
                <Icon size={16} />
            </div>
            <div className="label">{label}</div>
        </a>
    )
}

export default styled(SocialButton)`
    display: flex;
    justify-content: center;
    align-items: center;
    text-align: center;
    text-decoration: none;
    font-size: .9rem;
    user-select: none;
    padding: .25rem .5rem;
    border-radius: 0.25rem;
    color: white;
    background: hsla(203, 100%, 0%, .5);
    backdrop-filter: blur(.5rem);
    transition: all .1s linear;
    /* box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1); */
    border: 1px solid hsla(0,0%,0%,.25);
    cursor: pointer;

    &:hover {
        background: hsla(203, 100%, 33%, .75);
        border: 1px solid hsla(0,0%,0%,.5);
    }

    & .icon {
        display: flex;
        align-items: center;
    }

    & .label {
        margin-left: .5rem;
    }
`
