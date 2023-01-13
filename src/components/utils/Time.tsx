import { useEffect, useRef, useState } from "react";

export function MsFormat(timer: number) {
    let ms = ("00" + ((timer) % 1000)).slice(-3)
    let secs = ("0" + Math.floor((timer / 1000) % 60)).slice(-2)
    let mins = ("0" + Math.floor((timer / 60000) % 60)).slice(-2)
    let hours = ("0" + Math.floor((timer / 3600000) % 60)).slice(-2)

    return `${hours}:${mins}:${secs}:${ms}`
}
export class Time {
    time = 0
    ms = "000"
    secs = "00"
    mins = "00"
    hours = "00"
    value = "00:00:00:000"
    constructor(time: string | number) {
        if (typeof time === "string") {
            this.setTime(this.fromString(time).time)
        } else {
            this.setTime(time)
        }
    }

    add = (time: string | number) => {
        if (typeof time === "string") {
            let addedTime = this.fromString(time).time
            this.setTime(addedTime + this.time)
        } else {
            this.setTime(time + this.time)
        }
        return this
    }

    reset = () => this.setTime(0)

    setTime = (time: number) => {
        this.time = time

        this.ms = ("00" + ((time) % 1000)).slice(-3)
        this.secs = ("0" + Math.floor((time / 1000) % 60)).slice(-2)
        this.mins = ("0" + Math.floor((time / 60000) % 60)).slice(-2)
        this.hours = ("0" + Math.floor((time / 3600000) % 60)).slice(-2)

        this.value = this.toString()
        return this
    }

    toString = () => {
        return `${this.hours}:${this.mins}:${this.secs}:${this.ms}`
    }

    isValidTimeFormat = (value: string) => {
        const reg = /^(\d.*)(?::([0-5]?\d))?(?::([0-5]?\d))?(?::(\d{3}))$/gm
        return value.match(reg)
    }

    fromString = (value: string) => {
        let time = 0
        let splitted = value.split(":")

        if(this.isValidTimeFormat(value)) {
            let ms = splitted[3]
            let secs = splitted[2]
            let mins = splitted[1]
            let hours = splitted[0]

            time += parseInt(ms)
            time += parseInt(secs) * 1000
            time += parseInt(mins) * 60000
            time += parseInt(hours) * 3600000

            this.setTime(time)
        }
        return this
    }
}
