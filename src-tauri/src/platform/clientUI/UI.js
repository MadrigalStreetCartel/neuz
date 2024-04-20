function duplicateJsEvent(eventName, e) {
    switch (eventName) {
        case 'wheel':
        case 'mousewheel':
        case 'mousemove':
        case 'mousedown':
        case 'mouseup':
        case 'click':
        case 'contextmenu':
            return new MouseEvent(e.type, { clientX: e.clientX, clientY: e.clientY, button: e.button, buttons: e.buttons, movementX: e.movementX, movementY: e.movementY })
        case 'keydown':
        case 'keyup':
            return new KeyboardEvent(e.type, { key: e.key, code: e.code, keyCode: e.keyCode, which: e.which, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey })
    }
}

function duplicateEvent(eventName, parents = []) {
    const event = (e) => {
        e.preventDefault()
        e.stopPropagation()
        e = duplicateJsEvent(eventName, e)
        client.dispatchEvent(e)
    }
    parents.forEach(parent => {
        parent.addEventListener(eventName, event)
    })
}
function createOverlay(node, props = {}) {
    if (!node) {
        node = document.createElement('div')
    } else if (typeof node === "string") {
        node = document.createElement(node)
    }
    Object.keys(props).forEach(key => {
        node[key] = props[key]
    })
    return {
        shown: true,
        element: node,
        toggleShow() {
            if (this.shown) {
                this.hide()
            } else {
                this.show()
            }
        },
        hide() {
            if (!this.shown) return
            this.shown = false
            this.element.style.display = 'none'
        },
        show() {
            if (this.shown) return
            this.shown = true
            this.element.style.display = 'block'
        },
        mount(parent) {
            parent.appendChild(this.element)
        }
    }
}
