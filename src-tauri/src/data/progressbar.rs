use std::time::Instant;

#[derive(Debug, Default, Clone, Copy)]
pub struct ProgressBar {
    pub max_w: u32,
    pub value: u32,
    pub last_value: u32,
    pub last_update_time: Option<Instant>,
}

impl PartialEq for ProgressBar {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for ProgressBar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl ProgressBar {
    pub fn new(max_w: u32, value: u32) -> Self {
        Self {
            max_w,
            value,
            last_update_time: Some(Instant::now()),
            last_value: 100,
        }
    }

    pub fn reset_last_update_time(&mut self) {
        self.last_update_time = Some(Instant::now());
    }

    /// Update current value and max value of the progress bar returning true if the value has changed
    pub fn update_value(&mut self, (value, max_w): (u32, u32)) -> bool {
        let (old_max_w, old_value) = (self.max_w, self.value);

        if max_w != old_max_w {
            self.max_w = max_w;
        }
        if value != old_value {
            self.value = value;
            self.last_update_time = Some(Instant::now());
            true
        } else {
            false
        }
    }
}
