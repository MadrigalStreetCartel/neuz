import { profile } from "console"
import styled from "styled-components"

type Props = {
    className?: string,
    profileId: string
}

const ProfileDisplay = ({ className, profileId }: Props) => {
    return(
        <div className={className}>
           Profile: {profileId}
        </div>
    )
}

export default styled(ProfileDisplay)`
    background: hsla(203, 100%, 0%, .75);
    backdrop-filter: blur(.5rem);
    margin-top: 5px;
    padding: 5px;
    color: white;
`
