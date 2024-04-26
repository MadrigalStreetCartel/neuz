class Bounds {
    constructor(x, y, w, h) {
        this.x = x
        this.y = y
        this.w = w
        this.h = h
        this.element = document.createElement('div')
        this.element.style = `position: fixed; width: ${this.w}px; height: ${this.h}px; border: 1px solid violet;z-index: 9910;left: ${this.x}px;top: ${this.y}px; display: block;`

        let bounds_label = document.createElement('p')
        bounds_label.innerText = `${this.w}x${this.h}`
        bounds_label.style = `position: fixed; left: ${this.x}px;top: ${this.y - this.h-12}px; color: black; background-color: violet; padding: 2px; font-size: 12px;`

        this.element.appendChild(bounds_label)
    }

    draw(parent) {
        parent.appendChild(this.element)
    }

    remove() {
        this.element.remove()
    }
    equals(bounds) {
        return this.x === bounds.x && this.y === bounds.y && this.w === bounds.w && this.h === bounds.h
    }

    growBy(growValue) {
        this.w += growValue
        this.h += growValue
        this.x = this.x - growValue / 2
        this.y = this.y - growValue / 2
        this.element.style.width = `${this.w}px`
        this.element.style.height = `${this.h}px`
        this.element.style.left = `${this.x}px`
        this.element.style.top = `${this.y}px`
    }
}
