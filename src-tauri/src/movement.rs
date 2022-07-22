mod movement_accessor;
mod movement_coordinator;

pub use self::movement_accessor::MovementAccessor;
pub use self::movement_coordinator::{MovementCoordinator, Movement, MovementDirection, RotationDirection, ActionDuration};

pub mod prelude {
    pub use super::{Movement::*, MovementDirection as dir, RotationDirection as rot, ActionDuration as dur};
}

#[macro_export]
macro_rules! play {
    ($scheduler:expr => [ $($movement:expr,)+ $(,)? ]) => {{
        let scheduler: &MovementAccessor = &$scheduler;
        scheduler.schedule(|coordinator| {
            coordinator.play([$($movement),+]);
        });
    }};
}
