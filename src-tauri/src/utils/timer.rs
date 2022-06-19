use std::time::{Instant, Duration};

pub struct Timer {
    label: String,
    start: Instant,
}

impl Timer {
    pub fn start_new<S>(label: S) -> Timer
    where
        S: ToString,
    {
        Timer {
            label: label.to_string(),
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn report(&self) {
        println!("[{}] took {:?}", self.label, self.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.report();
    }
}
