import { useEffect, useRef, useState } from "react";
import { Time } from "./Time";

export const useStopWatch = (startCondition: boolean) => {
    const [started, setStarted] = useState(false);
    const [watch, setWatch] = useState<Time>(new Time(0))

/*     useEffect(()=> {
        watch = new Time(watch.time)
    }, []) */

    start()

    function start(shouldReset = false) {
        if(!started && startCondition) {
            shouldReset && watch.reset()
            setStarted(true)
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
            watch.setTime(watch.time + 10)
        }, 10);
      } else if (!started) {
        clearInterval(interval);
      }
      return () => clearInterval(interval);
    }, [started]);

    return {
        reset: watch.reset,
        start: start,
        stop: stop,
        started: started,
        watch: watch,
    }
  };
