let boundsOverlay = createOverlay(null, {
    style: 'position: fixed; top: 0; left: 0; z-index: 9900; background-color: rgba(0, 0, 0, 0.0); color: white; padding: 5px; font-size: 12px; height: 100%;width: 100%; display: block;',
    bounds: [],
    addBounds(x, y, w, h, growBy) {
        const bounds = new Bounds(x, y, w, h)
        if (growBy) bounds.growBy(growBy)
        if (this.bounds.some(b => b.equals(bounds))) return
        bounds.draw(this.element)
        this.bounds.push(bounds)
        return bounds
    },
    clear() {
        this.bounds.forEach(b => b.remove())
        this.bounds = []
    },
    removeBounds(bounds) {
        if (!(bounds instanceof Bounds)) return;
        const index = this.bounds.findIndex(b => b.equals(bounds))
        if (index === -1) return
        this.bounds[index].remove()
        this.bounds.splice(index, 1)
    }
})

let fpsElement = createOverlay("p", {
    style: `position: fixed; z-index: 9901; background-color: rgba(0, 0, 0, 0.6); color: white; padding: 5px; font-size: 12px; height: min-content; width: min-content; left: unset; top: unset; right:0; bottom:0;`,
    id: "fps-overlay",
    onMount: (self) => self.setFps(0),
    fpsHistory: [],
    setFps(fps) {
        this.fpsHistory.push(fps)
        const fpsAverage = this.fpsHistory.reduce((a, b) => a + b, 0) / this.fpsHistory.length
        this.element.innerText = `${fps}/avg:${fpsAverage.toFixed(2)}`
    },
})


let debugOverlay = createOverlay(null, {
    id: "debug-overlay",
    style: 'position: fixed; top: 0; left: 0; z-index: 9901; background-color: rgba(0, 0, 0, 0.0); color: white; padding: 5px; font-size: 12px; max-height: 100%;max-width: 100%; display:block;',
    boundsOverlay,
    fpsElement,

    hiddenOnScreenshot: ["boundsOverlay"],
    children: [fpsElement, boundsOverlay],

    hideOverlays() {
        this.hiddenOnScreenshot.forEach(overlay => this[overlay].hide())
    },
    showOverlays() {
        this.hiddenOnScreenshot.forEach(overlay => this[overlay].show())
    },
})


document.addEventListener('DOMContentLoaded', () => {
    console._log("init", document, document.readyState)
    debugOverlay.mount(document.body)
    // pass mouse events to the game
    let events = ['mousemove', 'mousedown', 'mouseup', 'click', 'contextmenu', 'wheel', 'mousewheel', 'keydown', 'keyup']
    events.forEach((eventName) => duplicateEvent(eventName, client, [debugOverlay.element/* , this.boundsOverlay.element */]))
})
