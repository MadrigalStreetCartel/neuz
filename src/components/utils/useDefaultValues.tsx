import { useRef } from "react";
import { AnyConfig } from "../../models/BotConfig";


const useDefaultValue = ( config: AnyConfig, onChange: (updatedConfig: AnyConfig) => void) => {
    //const [isShown, setIsShown] = useState(false)
    const refDefaultValues: any = useRef({
        'obstacle_avoidance_cooldown': 3000,
        'obstacle_avoidance_max_try': 3,

        'circle_pattern_rotation_duration': 30,

        'prevent_already_attacked': true,

        'passive_mobs_colors': [250, 250, 150],
        'passive_tolerence': 10,
        'aggressive_mobs_colors': [179, 23, 23],
        'aggressive_tolerence': 10,

        'min_mobs_name_width': 15,
        'max_mobs_name_width': 180,

        'interval_between_buffs': 2000,

        'inactivity_timeout': 0,
    })
    const startCheck  = useRef(false)

    if(!startCheck.current) {
        let newConfig = {...config}
        for (var key in refDefaultValues.current) {
            if(config[key] === null || key === "passive_mobs_colors" || key === "passive_tolerence") {
                newConfig[key] = refDefaultValues.current[key]
            }
        };
        onChange(newConfig)
        startCheck.current = true
    }

    function reset(keys: string[]) {
        let newConfig = {...config}
        for (var key in keys) {
            key = keys[key]
            newConfig[key] = refDefaultValues.current[key]
        }
        onChange(newConfig)
    }
    return {
        defaultValues: refDefaultValues.current,
        reset,
      };
}


export default useDefaultValue;
