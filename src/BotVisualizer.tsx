import styled from "styled-components"
import { listen, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import SlotBar from "./components/SlotBar"
import ConfigPanel from "./components/ConfigPanel"

export type FixedArray<TItem, TLength extends number> = [TItem, ...TItem[]] & { length: TLength }

type Bounds = {x: number, y: number, w: number, h: number}

type FrontendInfoModel = {
    enemy_bounds?: Bounds[],
    active_enemy_bounds?: Bounds,
    enemy_kill_count: number,
    is_attacking: boolean,
    is_running: boolean,
}

export type SlotType = "Unused" | "Food" | "PickupPet" | "AttackSkill" | "BuffSkill" | "Flying"
export type SlotModel = {
    slot_type: SlotType,
}
export type BotConfigModel = {
    change_id: number,
    is_running: boolean,
    on_demand_pet: boolean,
    use_attack_skills: boolean,
    stay_in_area: boolean,
    slots: FixedArray<SlotModel, 10>,
}

type Props = {
    className?: string,
}

const BotVisualizer = ({ className }: Props) => {
    const [imageData, setImageData] = useState({ data: '', width: 0, height: 0 })
    const [info, setInfo] = useState<FrontendInfoModel | null>(null);
    const [config, setConfig] = useState<BotConfigModel | null>(null);

    useEffect(() => {
        listen<string>('bot_visualizer_update', event => {
            const payload = event.payload as unknown as string[]
            const data = payload[0]
            const width = Number(payload[1])
            const height = Number(payload[2])
            setImageData({ data, width, height })
        })

        listen('frontend_info', event => {
            const payload = event.payload as FrontendInfoModel
            setInfo(payload)
        })

        listen('bot_config_s2c', event => {
            console.log(event.payload)
            const payload = event.payload as BotConfigModel
            setConfig(payload)
        })
    }, [])

    const handleToggle = () => {
        if (!config) return
        const newConfig = { ...config, is_running: !config.is_running }
        emit('bot_config_c2s', newConfig)
    }

    const handleSlotUpdate = (type: SlotType, index: number) => {
        if (!config) return
        const newSlots = [...config.slots]
        newSlots[index] = { ...newSlots[index], slot_type: type }
        const newConfig = { ...config, slots: newSlots as FixedArray<SlotModel, 10> }
        emit('bot_config_c2s', newConfig)
    }

    const handleConfigUpdate = (config: BotConfigModel) => {
        emit('bot_config_c2s', config)
    }

    return (
        <div className={className}>
            <div className="vstack">
                {info && (
                    <div className="info">
                        <div className="row">
                            <div>Running: {info.is_running ? '✅' : '❌'}</div>
                            <div>Kills: {info.enemy_kill_count}</div>
                            <div>Attacking: {info.is_attacking ? '✅' : '❌'}</div>
                        </div>
                    </div>
                )}
                {config && (
                    <>
                        <SlotBar slots={config.slots} onChange={handleSlotUpdate} />
                        <ConfigPanel config={config} onChange={handleConfigUpdate} />
                    </>
                )}
            </div>
            <div className="container" style={{ width: `${imageData.width}px`, height: `${imageData.height}px` }}>
                <img className="view" alt="" src={imageData.data} style={{ width: `${imageData.width}px`, height: `${imageData.height}px` }} />
                {/* {enemyTags.map(({ x, y }) => (
                    <div className="enemy-tag" style={{ left: `${x}px`, top: `${y}px` }} />
                ))}
                {enemyBounds.map(({ x, y, w, h }) => (
                    <div className="enemy-rect" style={{ left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px` }} />
                ))}
                {attackTargets.map(({ x, y }) => (
                    <div className="enemy-target" style={{ left: `${x}px`, top: `${y}px` }} />
                ))} */}
            </div>
            <div className="footer">
                <div className="btn" onClick={handleToggle}>{config?.is_running ? 'Stop' : 'Start'} Bot</div>
            </div>
        </div>
    )
}

export default styled(BotVisualizer)`
    width: 100vw;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
    overflow: hidden;

    & .vstack {
        position: fixed;
        top: 1rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
    }

    & .info {
        width: 50vw;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        padding: 1rem 2rem;
        gap: .5rem;
        color: white;
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        border-radius: 0.25rem;
        box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1);
        border: 1px solid hsl(0,0%,10%);
        z-index: 9999;

        &--slotbar {
            width: auto;
        }

        & .row {
            display: flex;
            align-items: center;
            justify-content: space-around;
            gap: 2rem;
        }
    }
    
    & .container {
        overflow: hidden;
        position: relative;
    }

    & .view {
        margin: 0;
        padding: 0;
        object-fit: fill;
    }

    & .enemy-tag {
        display: block;
        position: absolute;
        width: 4px;
        height: 4px;
        margin-top: -2px;
        margin-left: -2px;
        border-radius: 10rem;
        background-color: #ff0000;
        z-index: 1000;
    }

    & .enemy-rect {
        display: block;
        position: absolute;
        border: 2px solid #00ff00; 
        z-index: 2000;
    }

    & .enemy-target {
        display: block;
        position: absolute;
        width: 10px;
        height: 10px;
        margin-top: -5px;
        margin-left: -5px;
        border-radius: 10rem;
        background-color: white;
        z-index: 3000;
    }

    & .footer {
        display: flex;
        position: fixed;
        bottom: 0;
        left: 0;
        width: 100%;
        padding: 1rem;
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 9000;

        & .btn {
            cursor: pointer;
            user-select: none;
            padding: .25rem .5rem;
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
    }
`