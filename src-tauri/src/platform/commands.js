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


function isMob() {
    return document.body.style.cursor.indexOf('curattack') > 0
}

let mobClicker = {
    is_mob: false,
    waiting: null,
    queue: [],
    click: function (x, y) {
        dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
        dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))
        this.onClick(x, y)
    },
    add: function (x, y) {
        this.queue.push({ x, y })
    },
    clear: function () {
        this.waiting = null
        this.queue = []
        this.is_mob = false
    },
    onClick: function (x, y) {
        console.log('click', x, y)
    },
    onMove: function (x, y) {
        console.log('move', x, y)
    },
    run: function () {
        if (!this.waiting) {
            if (this.queue.length > 0) {
                this.is_mob = isMob()
                let pos = this.queue.shift()
                if (pos) {
                    dispatchEvent(new MouseEvent('mousemove', { clientX: pos.x, clientY: pos.y }))
                    this.onMove(pos.x, pos.y)
                    this.waiting = pos
                    setTimeout(() => {
                        this.waiting = null
                    }, 33)
                }
            }
        } else {
            this.is_mob = isMob()
            if (this.is_mob) {
                this.click(this.waiting.x, this.waiting.y)
                this.clear()
            }
        }
    }
}
document.addEventListener('DOMContentLoaded', () => {
    setInterval(() => {
        mobClicker.run()

    }, 0)
})


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
    function waitDuration(type) {
        if (duration) {
            after(duration, () => {
                dispatchEvent(new MouseEvent(type ?? 'mouseup', { clientX: x, clientY: y }))
            })
        } else if (type) {
            dispatchEvent(new MouseEvent(type, { key }))
        }
    }
    switch (type) {
        case 'move':
            dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))
            break;
        case 'press':
            dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
            waitDuration('mouseup')
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

