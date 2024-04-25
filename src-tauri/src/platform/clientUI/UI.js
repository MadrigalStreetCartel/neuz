function duplicateJsEvent(eventName, e) {
    switch (eventName) {
        case 'wheel':
        case 'mousewheel':
            return new WheelEvent(e.type, { clientX: e.clientX, clientY: e.clientY, deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey })
        case 'mousemove':
        case 'mousedown':
        case 'mouseup':
        case 'click':
        case 'contextmenu':
            return new MouseEvent(e.type, { clientX: e.clientX, clientY: e.clientY, button: e.button, buttons: e.buttons, movementX: e.movementX, movementY: e.movementY, metaKey: e.metaKey})
        case 'keydown':
        case 'keyup':
            return new KeyboardEvent(e.type, { key: e.key, code: e.code, keyCode: e.keyCode, which: e.which, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey })
    }
}

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
            this.parent = parent
            parent.appendChild(this.element)
            mounted = true
        },
        destroy() {
            if (!mounted) return
            for (let child of this.children) {
                child.destroy()
            }
            this.element.remove()
            mounted = false
        },
        ...props
    }
}
