import { useEffect, useState } from "react";

export const StopWatch = (running: boolean) => {
    const [time, setTime] = useState(0);
    useEffect(() => {
      let interval: string | number | NodeJS.Timeout | undefined;
      if (running) {
        interval = setInterval(() => {
          setTime((prevTime) => prevTime + 10);
        }, 10);
      } else if (!running) {
        clearInterval(interval);
      }
      return () => clearInterval(interval);
    }, [running]);

    let ms = ("0" + ((time / 10) % 100)).slice(-2)
    let secs = ("0" + Math.floor((time / 1000) % 60)).slice(-2)
    let mins = ("0" + Math.floor((time / 60000) % 60)).slice(-2)
    let hours = ("0" + Math.floor((time / 3600000) % 60)).slice(-2)

    return [hours,mins,secs]
};
