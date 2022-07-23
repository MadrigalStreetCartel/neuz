import styled from 'styled-components'

type Props = {
    label: React.ReactNode,
    item: React.ReactNode,
    layout?: "h" | "v"
}

const Label = styled.td`
    padding: .25rem 0;
`

const FlexLabel = styled.td`
    display: flex;
    justify-content: flex-start;
    align-items: flex-start;
    padding: .25rem 0;
`

const Item = styled.td`
    display: flex;
    justify-content: flex-end;
    align-items: flex-start;
    padding: .25rem 0;
`

const ConfigTableRow = ({ label, item, layout = "h" }: Props) => {
    return (
        <>
            {layout === "h" && (
                <Label>{label}</Label>
            )}
            {layout === "v" && (
                <FlexLabel>{label}</FlexLabel>
            )}
            <Item>{item}</Item>
        </>
    )
}

export default styled(ConfigTableRow)`
`
