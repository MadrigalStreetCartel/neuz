function duplicateJsEvent(eventName, e) {
    switch (eventName) {
        case 'wheel':
        case 'mousewheel':
            return new WheelEvent(e.type, { clientX: e.clientX, clientY: e.clientY, deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey })
        case 'mousemove':
        case 'mouseleave':
        case 'mousedown':
        case 'mouseup':
        case 'click':
        case 'contextmenu':
            return new MouseEvent(e.type, { clientX: e.clientX, clientY: e.clientY, button: e.button, buttons: e.buttons, movementX: e.movementX, movementY: e.movementY, metaKey: e.metaKey, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, relatedTarget: e.relatedTarget, screenX: e.screenX, screenY: e.screenY, pageX: e.pageX, pageY: e.pageY, offsetX: e.offsetX, offsetY: e.offsetY, x: e.x, y: e.y, layerX: e.layerX, layerY: e.layerY, which: e.which, fromElement: e.fromElement, toElement: e.toElement, })
        case 'keydown':
        case 'keyup':
            return new KeyboardEvent(e.type, { key: e.key, code: e.code, keyCode: e.keyCode, which: e.which, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey, repeat: e.repeat, location: e.location, charCode: e.charCode })

    }
}

let forwardEvents = ['wheel', 'mousewheel', 'mousemove', 'mouseleave', 'mousedown', 'mouseup', 'click', 'contextmenu', 'keydown', 'keyup']


/* HTMLElement.prototype._addEventListener = HTMLElement.prototype.addEventListener
HTMLElement.prototype.addEventListener = function (eventName, listener, options) {
    this._addEventListener(eventName, listener, options)
    if (this.tagName !== 'CANVAS') return
    console.log("addEventListener", { eventName, listener, options })
}; */

function duplicateEvent(eventName, parent, children = []) {
    const event = (e) => {
        e.preventDefault()
        e.stopPropagation()
        e = duplicateJsEvent(eventName, e)
        parent.dispatchEvent(e)
    }
    children.forEach(child => {
        child.addEventListener(eventName, event)
    })
}
function createOverlay(node, props = {}) {
    if (!node) {
        node = document.createElement('div')
    } else if (typeof node === "string") {
        node = document.createElement(node)
    }
    Object.keys(props).forEach(key => {
        // check if node has this property using hasProperty
        if (node[key] === undefined || key === 'children') return
        node[key] = props[key]
        delete props[key]
    })
    let enabled = props.enabled ?? true
    let mounted = false
    return {
        enabled: {
            get() {
                return enabled
            },
            set(value) {
                enabled = value
                if (value) {
                    this.mount()
                } else {
                    this.destroy()
                }
            }

        },
        _bindEvents: true,
        _areBoundEvents: false,
        shouldBindEvents: {
            get() {
                return this._bindEvents
            },
            set(value) {
                this._bindEvents = value
                if (value) {
                    this.bindEvents()
                } else {
                    this.unbindEvents()
                }
            }
        },
        shown: props.shown ?? true,
        element: node,
        parent: null,
        children: props.children ?? [],
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
        mount(parent = null) {
            if (!this.enabled || mounted) return
            props.onMount?.(this)
            console.log("mounting", this)
            for (let child of this.children) {
                child.mount(this.element)
            }
            if (this.shouldBindEvents) {
                this.bindEvents()
            }
            this.parent = parent
            parent.appendChild(this.element)
            mounted = true
        },
        destroy() {
            if (!mounted) return
            if (this.shouldBindEvents) {
                this.unbindEvents()
            }
            for (let child of this.children) {
                child.destroy()
            }
            this.element.remove()
            mounted = false
        },
        bindEvents(toElement = client) {
            if (this._areBoundEvents) return
            this._areBoundEvents = true
            this._eventListeners = {}
            for (let event of forwardEvents) {
                const listener = (e) => {
                    e.preventDefault()
                    e.stopPropagation()
                    e = duplicateJsEvent(event, e)
                    toElement.dispatchEvent(e)
                }
                this._eventListeners[event] = listener
                this.element.addEventListener(event, listener)
            }
        },
        unbindEvents() {
            if (!this._areBoundEvents) return
            this._areBoundEvents = false
            for (let event in this._eventListeners) {
                this.element.removeEventListener(event, this._eventListeners[event])
            }
        },
        ...props
    }
}
