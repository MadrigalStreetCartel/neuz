use std::time::Duration;

pub struct DateTime {}
impl DateTime {
    pub fn format_float(float: f32, precision: usize) -> f32 {
        format!("{:.prec$}", float, prec = precision)
            .parse::<f32>()
            .unwrap_or_default()
    }

    pub fn format_time(elapsed: Duration) -> String {
        let seconds = elapsed.as_secs() % 60;
        let minutes = (elapsed.as_secs() / 60) % 60;
        let hours = (elapsed.as_secs() / 60) / 60;

        let seconds_formatted = {
            if seconds < 10 {
                format!("0{}", seconds)
            } else {
                seconds.to_string()
            }
        };
        let minutes_formatted = {
            if minutes < 10 {
                format!("0{}", minutes)
            } else {
                minutes.to_string()
            }
        };
        let hours_formatted = {
            if hours < 10 {
                format!("0{}", hours)
            } else {
                hours.to_string()
            }
        };

        format!(
            "{}:{}:{}",
            hours_formatted, minutes_formatted, seconds_formatted
        )
    }
}
