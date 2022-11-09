import { useEffect, useRef, useState } from "react";

export class StopWatchValues {
    hours = "00"
    mins = "00"
    secs = "00"
    ms = "000"
    timer = 0

    constructor(timer: number) {
        this.update(timer)
    }

    toString = () => {
        return `${this.hours}:${this.mins}:${this.secs}:${this.ms}`
    }

    validStringFormat = (value: string) => {
        let splitted = value.split(":")
        if(splitted.length == 4 && splitted[0].length == 2 && splitted[1].length == 2 && splitted[2].length == 2 && splitted[3].length == 3) {
            return true
        }
        return false
    }

    fromString = (value: string) => {
        let time = 0
        let splitted = value.split(":")

        if(this.validStringFormat(value)) {
            let ms = splitted[3]
            let secs = splitted[2]
            let mins = splitted[1]
            let hours = splitted[0]

            time += parseInt(ms)
            time += parseInt(secs) * 1000
            time += parseInt(mins) * 60000
            time += parseInt(hours) * 3600000
        }

        this.timer = time
        return this
    }

    add = (watch: StopWatchValues) => {
        return new StopWatchValues(this.timer + watch.timer)
    }

    update = (timer: number) => {
        this.ms = ("00" + ((timer) % 1000)).slice(-3)
        this.secs = ("0" + Math.floor((timer / 1000) % 60)).slice(-2)
        this.mins = ("0" + Math.floor((timer / 60000) % 60)).slice(-2)
        this.hours = ("0" + Math.floor((timer / 3600000) % 60)).slice(-2)
        this.timer = timer
    }
}
export const useStopWatch = () => {
    const time = useRef(0);
    const [started, setStarted] = useState(false);
    const watch = useRef(new StopWatchValues(time.current))

    function reset() {
        time.current = 0;
    }

    function start(startCondition: boolean, shouldReset = false) {
        if(!started && startCondition) {
            shouldReset && reset()
            setStarted(true);
        }else if(started && !startCondition) {
            stop()
        }
    }

    function stop() {
        started && setStarted(false);
    }

    useEffect(() => {
      let interval: string | number | NodeJS.Timeout | undefined;
      if (started) {
        interval = setInterval(() => {
            time.current = time.current + 10
            watch.current.update(time.current)
        }, 10);
      } else if (!started) {
        clearInterval(interval);
      }
      return () => clearInterval(interval);
    }, [started]);

    return {
        reset: reset,
        start: start,
        stop: stop,
        started: started,
        watch: watch.current,
    }
  };
