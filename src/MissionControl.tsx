import styled from "styled-components"
import { listen, emit } from '@tauri-apps/api/event'
import { useEffect, useState } from "react"
import { isNil } from 'lodash'

import TabControl from "./components/TabControl"
import Tab from "./components/Tab"

import ImageFarm from './assets/btn_leveling.png'
import ImageSupport from './assets/btn_full_support.png'
import ImageShout from './assets/btn_shout.png'

import { BotConfigModel, ModeModel } from './models/BotConfig'
import FarmingConfig from "./components/behaviors/FarmingConfig"
import SupportConfig from "./components/behaviors/SupportConfig"
import ShoutConfig from "./components/behaviors/ShoutConfig"
import Footer from "./components/Footer"
import { FrontendInfoModel } from "./models/FrontendInfo"
import { invoke } from "@tauri-apps/api"
import { useStopWatch } from "./components/utils/StopWatch"

//type Bounds = {x: number, y: number, w: number, h: number}

type Props = {
    className?: string,
    currentVersion?: string,
    lastVersion?: string,
}

const MissionControl = ({ className, lastVersion, currentVersion }: Props) => {
    //const [imageData, setImageData] = useState({ data: '', width: 0, height: 0 })
    const [info, setInfo] = useState<FrontendInfoModel | null>(null);
    const [config, setConfig] = useState<BotConfigModel | null>(null);

    useEffect(() => {
        listen<string>('bot_visualizer_update', event => {
        /*  const payload = event.payload as unknown as string[]
            const data = payload[0]
            const width = Number(payload[1])
            const height = Number(payload[2])
            setImageData({ data, width, height }) */
        })

        listen('bot_info_s2c', event => {
            const payload = event.payload as FrontendInfoModel
            setInfo(payload)
        })

        listen('bot_config_s2c', event => {
            console.log(event.payload)
            const payload = event.payload as BotConfigModel
            setConfig(payload)
        })
    }, [])

    const setRunningToggle = () => {
        if (!config) return
        const newConfig = { ...config, is_running: !config.is_running }
        emit('bot_config_c2s', newConfig)
    }

    const handleToggle = () => {
        if (!config) return
        const newConfig = { ...config }
        newConfig.farming_config.is_stop_fighting = !newConfig.farming_config.is_stop_fighting
        emit('bot_config_c2s', newConfig)
    }


    const handleTabSelect = (mode: ModeModel) => {
        const newConfig = { ...config, mode }
        emit('bot_config_c2s', newConfig)
    }

    const makeConfigUpdater = (key: string) => <T,>(patchedConfig: T) => {
        const newConfig = { ...config, [key]: patchedConfig }
        emit('bot_config_c2s', newConfig)
    }

    const [lightMode, setLightMode] = useState(false)
    const minSizeFarm = [250, 200]
    const minSize = [250, 160]
    const toogleLightMode = () => invoke("toggle_main_size", {size: config?.mode === "Farming"? minSizeFarm : minSize}).then((value) => setLightMode(value as boolean))

    const farmingState = config?.is_running? info?.is_running? !info?.is_alive? "dead" : info.is_attacking? "fighting" : "searching" : "ready" : "idle"
    const farmStopWatch = useStopWatch(farmingState === "searching" || farmingState === "fighting")

    const supportState = config?.is_running? info?.is_running? !info?.is_alive? "dead" : "healing" : "ready" : "idle"
    const suppStopWatch = useStopWatch(supportState === "healing")
    return (
        <div className={className}>
            {!lightMode && <div className={`vstack`}>
                {config && (
                    <>
                        <TabControl activeMode={config.mode} onSelect={handleTabSelect}>
                            <Tab mode="Farming" image={ImageFarm} />
                            <Tab mode="Support" image={ImageSupport} />
                            <Tab mode="AutoShout" image={ImageShout} />
                        </TabControl>
                        <div className="config-container">
                            {config?.mode === 'Farming' && (<FarmingConfig botState={farmingState} botStopWatch={farmStopWatch.watch} running={config.is_running} isCurrentMode={config.mode === "Farming"} info={info} config={config.farming_config} onChange={makeConfigUpdater('farming_config')} />)}
                            {config?.mode === 'Support' && (<SupportConfig botState={supportState} botStopWatch={suppStopWatch.watch} running={config.is_running} isCurrentMode={config.mode === "Support"} info={info} config={config.support_config} onChange={makeConfigUpdater('support_config')} />)}
                            {config?.mode === 'AutoShout' && (<ShoutConfig config={config.shout_config} onChange={makeConfigUpdater('shout_config')} />)}
                        </div>
                    </>
                )}


                <div className="footer">
                    <div className="sm footerCombinedBtn">
                        {!isNil(config?.mode) && <div className="btn sm" onClick={toogleLightMode}>Compact</div>}
                        {!isNil(config?.mode) && <div className="btn sm" onClick={() => {invoke("focus_client")}}>Focus</div>}
                    </div>
                    {!isNil(config?.mode) && <div className="btn" onClick={setRunningToggle}>{config?.is_running ? 'Disengage' : 'Engage'}</div>}
                </div>
                <Footer currentVersion={currentVersion} lastVersion={lastVersion} />
            </div>}

            {lightMode &&
                <div className="lightmode">
                       {config?.mode !== "AutoShout" && <div className={"stateDisplay"}>
                            State: {config?.mode === "Farming"? farmingState : supportState}
                        </div>}
                    <div className="btn sm" id="back" onClick={toogleLightMode}>{"<-"}</div>
                    <div className="btn sm" onClick={() => {invoke("focus_client")}}>Focus</div>
                    {config?.mode === "Farming" && <div className="btn sm" onClick={handleToggle}>
                    {config?.farming_config.is_stop_fighting? "🛑" : "✅" }🎯
                    </div>}
                    <div className="btn sm" onClick={setRunningToggle}>{config?.is_running ? 'Disengage' : 'Engage'}</div>
                </div>
            }
            {/* <div className="container" style={{ width: `${imageData.width}px`, height: `${imageData.height}px` }}>
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
            </div> */}
        </div>
    )
}

export default styled(MissionControl)`
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    overflow: auto;
    padding: 0.25rem 0;
    padding-bottom: 0;

    & .stateDisplay {
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        margin-bottom: 5px;
        padding: 5px;
        width: 70%;
        color: white;
    }

    & .vstack {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        height: 100%;
    }
    & .lightmode {
        justify-content: center;
        align-items: center;
        text-align: -webkit-center;
        margin-bottom: 10px;

        & #back {
            position: fixed;
            top: 5px;
            left: 5px;
            width: 40px;
            font-size: 15px;
        }
    }

    & .info {
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        padding: .5rem 1rem;
        gap: .5rem;
        color: white;
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        border-radius: 0.25rem;
        box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1);
        border: 1px solid hsl(0,0%,10%);
        z-index: 0;
        width: calc(min(500px, max(250px, 50vw)));


        &--slotbar {
            width: auto;
        }

        & .row {
            display: flex;
            align-items: center;
            justify-content: space-around;
        }
    }

    & .stats {
        & .row {
            color: white;
            display: flex;
            align-items: center;
            justify-content: space-around;
            gap: 3rem;
        }
    }

    & .container {
        overflow: hidden;
        position: relative;
    }

    & .config-container {
        width: 100%;
        max-width: 90vw;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.25rem;
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
        width: 100%;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        margin-top: auto;

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

    & .footerCombinedBtn {

        display: flex;
        flex-direction: row;
        & .btn {
            width: 50%;
        }
    }
`
