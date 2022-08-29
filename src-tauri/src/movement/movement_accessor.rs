use parking_lot::Mutex;

//use crate::platform::PlatformAccessor;

use super::MovementCoordinator;

pub struct MovementAccessor/*<'a>*/ {
    coordinator: Mutex<MovementCoordinator/*<'a>*/>,
}

impl<'a> MovementAccessor/*<'a>*/ {
    pub fn new(/*platform: &'a PlatformAccessor<'a>*/) -> Self {
        Self {
            coordinator: Mutex::new(MovementCoordinator::new(/*platform*/)),
        }
    }

    pub fn schedule<F>(&self, func: F)
    where
        F: Fn(&mut MovementCoordinator),
    {
        let mut coordinator = self.coordinator.lock();
        func(&mut coordinator);
    }
}
