import { useEffect, useRef, useState } from "react";

class StopWatchValues {
    hours = "00"
    mins = "00"
    secs = "00"
    ms = "00"
    timer = 0

    constructor(timer: number) {
        this.ms = ("0" + ((timer / 10) % 100)).slice(-2)
        this.secs = ("0" + Math.floor((timer / 1000) % 60)).slice(-2)
        this.mins = ("0" + Math.floor((timer / 60000) % 60)).slice(-2)
        this.hours = ("0" + Math.floor((timer / 3600000) % 60)).slice(-2)
        this.timer = timer
    }

    add = (watch: StopWatchValues) => {
        return new StopWatchValues(this.timer + watch.timer)
    }
}
export const StopWatch = () => {
    const time = useRef(0);
    const [started, setStarted] = useState(false);

    function reset() {
        time.current = 0;
    }

    function start(startCondition: boolean,shouldReset = false) {
        if(!started && startCondition) {
            shouldReset && reset()
            setStarted(true);
        }if(started && !startCondition) {
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
        watch: new StopWatchValues(time.current),
    }
  };
