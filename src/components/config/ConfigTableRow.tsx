import styled from 'styled-components'

type Props = {
    label: React.ReactNode,
    item: React.ReactNode,
    layout?: "h" | "v",
    disabled?: boolean,
}

type ItemProps = {
    disabled: boolean,
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

const Item = styled.td<ItemProps>`
    display: flex;
    justify-content: flex-end;
    align-items: flex-start;
    padding: .25rem 0;


    &[disabled] {
        opacity: .5;
        filter: grayscale(100%);
        pointer-events: none;
    }
`

const ConfigTableRow = ({ label, item, layout = "h", disabled = false }: Props) => {
    return (
        <>
            {layout === "h" && (
                <Label>{label}</Label>
            )}
            {layout === "v" && (
                <FlexLabel>{label}</FlexLabel>
            )}
            <Item disabled={disabled}>{item}</Item>
        </>
    )
}

export default styled(ConfigTableRow)`

`
