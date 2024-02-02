canvas = document.querySelector('canvas');
chatInput = document.querySelector('input');
createKeyBoardEvent = (key, type) => {
    return new KeyboardEvent(type, { key: key });
}

createMouseEvent = (type, x, y) => {
    return new MouseEvent(type, { clientX: x, clientY: y });
}

dispatchEventToCanvas = (type, ...args) => {
    let event = null;
    type = type.toLowerCase();
    if (type === 'keyboard') {
        event = createKeyBoardEvent(...args);
    } else if (type === 'mouse') {
        event = createMouseEvent(...args);
    }

    if (!event) return;
    canvas.dispatchEvent(event);
}

releaseKey = function (key) {
    dispatchEventToCanvas("keyboard", key, 'keyup');
}
pressKey = function (key) {
    dispatchEventToCanvas("keyboard", key, 'keydown');
    releaseKey(key);
}

holdKey = function (key, duration = null) {
    pressKey(key);

    if (!duration) return;

    setTimeout(() => {
        releaseKey(key);
    }, duration);

}

sendKey = function (key, type, duration = null) {
    type = type.toLowerCase();
    if (type === 'press') {
        pressKey(key);
    } else if (type === 'hold') {
        holdKey(key, duration);
    } else if (type === 'release') {
        releaseKey(key);
    }
}
let currentSlotBar = null;
sendSlotBar = function (slotBarIndex) {
    if (currentSlotBar === slotBarIndex) return;
    currentSlotBar = slotBarIndex;
    sendKey("F" + slotBarIndex, "press");
}
sendSlot = function (slotIndex) {
    sendKey(slotIndex, "press");
}
mouseMove = function (x, y) {
    dispatchEventToCanvas("mouse", "mousemove", x, y);
}
sendMouse = function (type, x, y) {
    type = type.toLowerCase();
    if (type === 'click') {
        mouseClick(x, y);
    } else if (type === 'move') {
        mouseMove(x, y);
    }
}
mouseClick = function (x, y) {
    dispatchEventToCanvas("mouse", "mousedown", x, y);
    dispatchEventToCanvas("mouse", "mouseup", x, y);
}

isCursorAttack = () => {
    return document.body.style.cursor.indexOf('curattack') > 0
}

drawHTMLSquareOverlay = (color, x, y, duration = null, width = 25, height = 25) => {
    let square = document.createElement('div');
    x -= width / 2;
    y -= height / 2;
    square.style.position = 'absolute';
    square.style.left = x + 'px';
    square.style.top = y + 'px';
    square.style.width = width + 'px';
    square.style.height = height + 'px';
    square.style.backgroundColor = color;
    square.style.zIndex = '9999';
    document.body.appendChild(square);
    if (duration) {
        setTimeout(() => {
            square.remove();
        }, duration);
    }
}
mobClickDelay = 10.0;
mobClick = function (x, y, debug = true) {
    mouseMove(x, y);
    setTimeout(() => {
        if (isCursorAttack()) {
            mouseClick(x, y);
            if (debug) {
                drawHTMLSquareOverlay('rgba(0,255,0,0.5)', x, y, 500)
                drawHTMLSquareOverlay('black', x, y, 500, 5, 5)
            }
        } else {
            if (debug) {
                drawHTMLSquareOverlay('rgba(255,0,0,0.5)', x, y, 500)
                drawHTMLSquareOverlay('black', x, y, 500, 5, 5)
            }
        } /* else {
            alert("Cursor is not attack, ", mobClickDelay, " delay added");
            mobClickDelay += 0.1;
            if (mobClickDelay > 25.0) {
                mobClickDelay = 0.0;
            }
        } */
    }, mobClickDelay)
}

writeMessage = function (message) {
    chatInput.value = message;
    chatInput.select();
}



