import styled from 'styled-components'
import { WebviewWindow } from '@tauri-apps/api/window'

import FlyffLogo from './assets/logo.png'
import LauncherBackground from './assets/launcher_background_ice.png'
import { useEffect, useState } from 'react'
import BotVisualizer from './BotVisualizer'

type Props = {
    className?: string,
}

type NewsItem = {
    type: 'news' | 'event',
    link: string,
    title: string,
    date: string,
}

const fetchNews: () => Promise<[NewsItem[], NewsItem[]]> = async () => {
    const url = `https://cors.splitty.me/https://universe.flyff.com/news`
    const resp = await fetch(url)
    const html = await resp.text()
    const doc = new DOMParser().parseFromString(html, 'text/html')
    const elNewsBase = Array.from(doc.querySelectorAll('#nav-1 #recent-announcements a'))
    const elEventsBase = Array.from(doc.querySelectorAll('#nav-2 #recent-announcements a'))
    const mapItems = (elBase: Element[], type: NewsItem["type"]) => elBase.map(el => {
        const link = el.getAttribute('href');
        const newsBlock = el.querySelector('.news-item');
        const date = newsBlock?.querySelector('p');
        const title = newsBlock?.querySelector('h4');
        if (!link || !newsBlock || !date || !title) return null;
        const item: NewsItem = {
            type,
            link: link?.replace(/:\d+/, '') || '',
            title: title.textContent || '',
            date: date.textContent || '',
        }
        return item
    }).filter(news => news !== null) as NewsItem[];
    const news = mapItems(elNewsBase, 'news')
    const events = mapItems(elEventsBase, 'event')
    return [news, events];
}

const Launcher = ({ className }: Props) => {
    const [isLaunched, setIsLaunched] = useState(false)
    const [recentNews, setRecentNews] = useState<NewsItem[][]>([])

    useEffect(() => {
        fetchNews().then(setRecentNews)
    }, [])

    const launch = () => {
        const webview = new WebviewWindow(`client`, {
            title: 'Flyff Universe',
            url: 'https://universe.flyff.com/play'
        })
    
        webview.once('tauri://created', function () {
            webview.show()
            setIsLaunched(true)
        })

        webview.once('tauri://close-requested', function () {
            webview.close()
            setIsLaunched(false)
        })

        webview.once('tauri://closed', function () {
            setIsLaunched(false)
        })
    }

    return (
        <div className={className}>
            {!isLaunched && (
                <div className="container">
                    <img className="logo" alt="Flyff Universe Logo" src={FlyffLogo} />
                    <div className="news">
                        {recentNews.map(newsBlock => (
                            <>
                                {newsBlock.map(({ type, link, title, date }) => (
                                    <div className="news-item" key={link}>
                                        <div className="badge">{type}</div>
                                        <a rel="noreferrer" href={link} target="_blank">{title}</a>
                                        <div>{date}</div>
                                    </div>
                                ))}
                            </>
                        ))}
                    </div>
                    <div className="btn" onClick={launch}>Play</div>
                </div>
            )}
            {isLaunched && (
                <BotVisualizer />
            )}
        </div>
    )
}

export default styled(Launcher)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100vw;
    height: 100vh;
    background-image: url(${LauncherBackground});
    background-attachment: fixed;
    background-repeat: no-repeat;
    background-position: center center;
    background-size: cover;
    overflow: hidden;

    & .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: space-around;
        height: 100vh;
    }

    & .news {
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        padding: 1rem;
        gap: .5rem;
        color: white;
        background: hsla(203, 100%, 0%, .75);
        backdrop-filter: blur(.5rem);
        border-radius: 0.25rem;
        box-shadow: 0 .1rem .1rem 0 hsla(0,0%,0%,1);
        border: 1px solid hsl(0,0%,10%);

        & .news-item {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 1rem;

            & .badge {
                padding: .25rem .5rem;
                text-align: center;
                background: hsla(0,0%,0%,.5);
                border-radius: .25rem;
            }

            a {
                color: white;
            }

            & > *:last-child {
                margin-left: auto;
                opacity: 0.75;
            }
        }
    }

    & .logo {
        height: 25vh;
        max-height: 250px;
        object-fit: scale-down;
    }

    & .btn {
        cursor: pointer;
        user-select: none;
        padding: 1rem 1rem;
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
`