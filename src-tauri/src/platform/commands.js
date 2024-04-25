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

function drawBounds(x, y, w, h) {
    if (!DEBUG) return
    let bounds = debugOverlay.boundsOverlay.addBounds(x, y, w, h, 4)

    setTimeout(() => {
        debugOverlay.boundsOverlay.removeBounds(bounds)
    }, 1000)

}


function isMob() {
    return document.body.style.cursor.indexOf('curattack') > 0
}
let clickPos = null

setInterval(() => {
    if (clickPos !== null && isMob()) {
        dispatchEvent(new MouseEvent('mousedown', { clientX: clickPos.x, clientY: clickPos.y }))
        dispatchEvent(new MouseEvent('mouseup', { clientX: clickPos.x, clientY: clickPos.y }))
        addTargetMarker('green', clickPos.x, clickPos.y)
        clickPos = null
    }
}, 0)

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
function mouseEvent(type, x, y, { checkMob = false, delay = 100, duration } = {}) {
    if (checkMobTimeout) {

        clearTimeout(checkMobTimeout)
        checkMobTimeout = null
        clickPos = null
    }
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
            dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))

            if (checkMob) {
                clickPos = { x, y }
                checkMobTimeout = setTimeout(() => {
                    clickPos = null
                }, delay)
            } else if (!checkMob) {
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

