function addTargetMarker(color = 'red', x = 0, y = 0,) {
    if (!DEBUG) return
    const targetMarker = document.createElement('div')
    const targetMarkerStyle = `position: fixed; width: 4px; height: 4px; background-color: ${color}; border-radius: 50%;z-index: 9999;left: ${x - 2}px;top: ${y - 2}px;`
    targetMarker.style = targetMarkerStyle
    debugOverlay.element.appendChild(targetMarker)

    setTimeout(() => {
        targetMarker.remove()
    }, 1000)
}

function drawBounds(x, y, w, h, duration = 1000) {
    if (!DEBUG) return
    let bounds = debugOverlay.boundsOverlay.addBounds(x, y, w, h, 4)

    setTimeout(() => {
        debugOverlay.boundsOverlay.removeBounds(bounds)
    }, duration)

}

let drawnBounds = [];
function drawGroups(bounds = [], duration = 1000) {
    if (!DEBUG) return
    clearDrawnBounds()
    let color = 'violet';
    for (let bound of bounds) {
        const { x, y, w, h } = bound
        const targetMarker = document.createElement('div')
        const targetMarkerStyle = `position: fixed; width: ${w}px; height: ${h}px; border: 1px solid ${color};z-index: 9999;left: ${x}px;top: ${y}px;`
        targetMarker.style = targetMarkerStyle
        document.body.appendChild(targetMarker)
        drawnBounds.push(targetMarker)
        i++
    }
    setTimeout(() => {
        clearDrawnBounds()
    }, duration)
}

function clearDrawnBounds() {
    drawnBounds.forEach((bound) => {
        bound.remove()
    })
    drawnBounds = [];
}


function isMob() {
    return document.body.style.cursor.indexOf('curattack') > 0
}

let max_tries = 2
let current_tries = 0

let mobClicker = {
    waiting: null,
    queue: [],
    click(x, y) {
        dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
        dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))
        this.onClick(x, y)
    },
    add(x, y) {
        this.queue.push({ x, y })
        this.run()
    },
    clear() {
        this.waiting = false
        this.queue = []
    },
    onClick(x, y) {
        console.log('click', x, y)
    },
    onMove(x, y) {
        console.log('move', x, y)
    },
    mobClick(x, y) {
        if (!this.waiting) return
        //_console.log('Mob click', x, y, 'tries:', current_tries, 'max:', max_tries, 'isMob:', isMob());
        if (current_tries >= max_tries) {
            this.waiting = false
            current_tries = 0
            mobClicker.run();
            return
        }
        current_tries++
        if (isMob()) {
            mouseEvent('press', x, y)
            this.waiting = false
            current_tries = 0
            this.clear();
            return;
        }
        requestAnimationFrame(() => this.mobClick(x, y));
    },
    run() {
        if (!this.waiting) {
            if (this.queue.length > 0) {
                let pos = this.queue.shift()
                if (pos) {
                    let { x, y } = pos;
                    dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))
                    this.onMove(x, y)
                    this.waiting = true
                    requestAnimationFrame(() => this.mobClick(x, y));
                    return;
                }
            }
        }

    }
}


function sendSlot(slotBarIndex, slotIndex) {
    keyboardEvent('press', `F${slotBarIndex + 1}`)
    keyboardEvent('press', slotIndex)
}

function setInputChat(text) {
    input.value = text
    input.select()
}

function dispatchEvent(event) {
    return client.dispatchEvent(event)
}

function after(duration = 0, callback) {
    setTimeout(callback, duration)
}

let checkMobTimeout = null;
function mouseEvent(type, x, y, { checkMob = false, delay = 50, duration } = {}) {
    function waitDuration(type, props = {}) {
        if (duration) {
            after(duration, () => {
                dispatchEvent(new MouseEvent(type ?? 'mouseup', { clientX: x, clientY: y }))
            })
        } else if (type) {
            dispatchEvent(new MouseEvent(type, props))
        }
    }
    switch (type) {
        case 'move':
            dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))
            break;
        case 'press':
            dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
            waitDuration('mouseup', { clientX: x, clientY: y })
            break;
        case 'hold':
            dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
            waitDuration()
            break;
        case 'release':
            dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))
            break;
        case 'moveClick':

            if (checkMob) {
                mobClicker.add(x, y)
            } else if (!checkMob) {
                dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))
                addTargetMarker('blue', x, y)
                dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
                dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))
            }
            break;
    }
}
function keyboardEvent(keyMode, key, duration = null) {
    function waitDuration(type) {
        if (duration) {
            setTimeout(() => {
                dispatchEvent(new KeyboardEvent(type ?? 'keyup', { key }))
            }, duration)
        } else if (type) {
            dispatchEvent(new KeyboardEvent(type, { key }))
        }
    }
    switch (keyMode) {
        case 'press':
            dispatchEvent(new KeyboardEvent('keydown', { key }))
            waitDuration('keyup')
            break;
        case 'hold':
            dispatchEvent(new KeyboardEvent('keydown', { key }))
            waitDuration()
            break;
        case 'release':
            dispatchEvent(new KeyboardEvent('keyup', { key }))
            break;

    }
}

