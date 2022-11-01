import { useRef } from "react";
import { AnyConfig } from "../../models/BotConfig";

export const defaultValuesChecker = ( config: AnyConfig, defaultValues: any, onChange: (updatedConfig: AnyConfig) => void ) => {
    let default_values_checked = useRef(false)
    if(!default_values_checked.current) {
        let newConfig = {...config}
        for (var key in defaultValues) {
            if(!config[key]) {
                newConfig[key] = defaultValues[key]
            }
        };
        onChange(newConfig)
        default_values_checked.current = true
    }
}
