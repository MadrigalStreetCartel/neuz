let mob_clicker_interval = null
const client = document.querySelector('canvas')
const input = document.querySelector('input')
const DEBUG = true && $env.DEBUG
function addTargetMarker(color = 'red', x = 0, y = 0,) {
    if (!DEBUG) return
    const targetMarker = document.createElement('div')
    const targetMarkerStyle = `position: fixed; width: 3px; height: 3px; background-color: ${color}; border-radius: 50%;z-index: 9999;left: ${x}px;top: ${y}px;`
    targetMarker.style = targetMarkerStyle
    document.body.appendChild(targetMarker)

    setTimeout(() => {
        targetMarker.remove()
    }, 1000)

    return targetMarker
}

function drawBounds(x, y, w, h) {
    if (true || !DEBUG) return
    const targetMarker = document.createElement('div')
    const targetMarkerStyle = `position: fixed; width: ${w}px; height: ${h}px; border: 1px solid violet;z-index: 9999;left: ${x}px;top: ${y}px;`
    targetMarker.style = targetMarkerStyle
    document.body.appendChild(targetMarker)

    // adds a div on top and display width and height
    const targetMarkerText = document.createElement('div')
    const targetMarkerTextStyle = `position: fixed; color: violet; z-index: 9999; left: ${x}px; top: ${(y - h) - 10}px;`
    targetMarkerText.style = targetMarkerTextStyle
    targetMarkerText.innerHTML = `${w}x${h}`
    document.body.appendChild(targetMarkerText)



    setTimeout(() => {
        targetMarkerText.remove()
        targetMarker.remove()

    }, 1000)
    return targetMarker
}

function isMob() {
    return document.body.style.cursor.indexOf('curattack') > 0
}
let try_check = false
let coords = { x: 0, y: 0 }
let tried_target = null
setInterval(() => {
    if (try_check && isMob()) {
        tried_target.remove()
        mouseEvent('press', coords.x, coords.y)
        addTargetMarker('green', coords.x, coords.y)
        try_check = false
        setTimeout(() => {
            mouseEvent('move', 800 / 2, 600 / 2)
        }, 50)
    }
},0)


function dispatchEvent(event) {
    return client.dispatchEvent(event)
}

function after(duration = 0, callback) {
    setTimeout(callback, duration)
}

let checkMobTimeout = null;
function mouseEvent(type, x, y, { checkMob = false, delay = 20, duration } = {}) {
    if (checkMobTimeout) {

        clearTimeout(checkMobTimeout)
        checkMobTimeout = null
    }
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
            waitDuration('mouseup', { clientX: x, clientY: y })
            break;
        case 'release':
            dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))
            break;
        case 'moveClick':
            dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y }))
            tried_target = addTargetMarker('orange', x, y)

            if (checkMob) {
                try_check = true
                coords = { x, y }
                setTimeout(() => {
                    try_check = false
                }, 33)
                /*  if (mob_clicker_interval) {
                     clearInterval(mob_clicker_interval)
                 }

                 mob_clicker_interval = setInterval(() => {
                     let to = setTimeout(() => {
                         if (mob_clicker_interval) {
                             clearInterval(mob_clicker_interval)
                         }
                     }, 10)
                     if (isMob()) {
                         tried_target.remove()
                         mouseEvent('press', x, y)
                         addTargetMarker('green', x, y)
                         clearTimeout(to)
                         clearInterval(mob_clicker_interval)
                         setTimeout(() => {
                             mouseEvent('move', 800 / 2, 600 / 2)
                         }, 30)
                     }
                 }, 0) */
                /*  checkMobTimeout = setTimeout(() => {
                     if (isMob()) {
                         dispatchEvent(new MouseEvent('mousedown', { clientX: x, clientY: y }))
                         dispatchEvent(new MouseEvent('mouseup', { clientX: x, clientY: y }))

                         addTargetMarker('green', x, y)
                     } else {
                         //dispatchEvent(new MouseEvent('mousemove', { clientX: 0, clientY: 0 }))
                         addTargetMarker('red', x, y)
                     }
                 }, delay) */
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

function sendSlot(slotBarIndex, slotIndex) {
    keyboardEvent('press', `F${slotBarIndex + 1}`)
    keyboardEvent('press', slotIndex)
}

function setInputChat(text) {
    input.value = text
    input.select()
}
