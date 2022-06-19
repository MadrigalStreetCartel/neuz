import styled from "styled-components"
import { listen } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"

type Props = {
    className?: string,
}

const BotVisualizer = ({ className }: Props) => {
    const [imageData, setImageData] = useState({ data: '', width: 0, height: 0 })
    const [enemyTags, setEnemyTags] = useState<{x: number, y: number}[]>([])
    const [enemyBounds, setEnemyBounds] = useState<{x: number, y: number, w: number, h: number}[]>([])
    const [attackTargets, setAttackTargets] = useState<{x: number, y: number}[]>([])

    useEffect(() => {
        listen<string>('bot_visualizer_update', event => {
            const payload = event.payload as unknown as string[]
            const data = payload[0]
            const width = Number(payload[1])
            const height = Number(payload[2])
            setImageData({ data, width, height })
        })

        listen('bot_visualizer_enemy_tags', event => {
            const payload = event.payload as number[][]
            const tags = payload.map(([x, y]) => ({ x, y }))
            setEnemyTags(tags)
        })

        listen('bot_visualizer_enemy_bounds', event => {
            const payload = event.payload as {x: number, y: number, w: number, h: number}[]
            setEnemyBounds(payload)
        })

        listen('bot_visualizer_target_points', event => {
            const payload = event.payload as number[][]
            const targets = payload.map(([x, y]) => ({ x, y }))
            setAttackTargets(targets)
        })
    }, [])

    return (
        <div className={className}>
            <div className="container" style={{ width: `${imageData.width}px`, height: `${imageData.height}px` }}>
                <img className="view" alt="" src={imageData.data} style={{ width: `${imageData.width}px`, height: `${imageData.height}px` }} />
                {enemyTags.map(({ x, y }) => (
                    <div className="enemy-tag" style={{ left: `${x}px`, top: `${y}px` }} />
                ))}
                {enemyBounds.map(({ x, y, w, h }) => (
                    <div className="enemy-rect" style={{ left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px` }} />
                ))}
                {attackTargets.map(({ x, y }) => (
                    <div className="enemy-target" style={{ left: `${x}px`, top: `${y}px` }} />
                ))}
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

    & .container {
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
`