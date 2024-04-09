mod movement_accessor;
mod movement_coordinator;

pub use self::{
    movement_accessor::MovementAccessor,
    movement_coordinator::{
        ActionDuration, Movement, MovementCoordinator, MovementDirection, RotationDirection,
    },
};

pub mod prelude {
    pub use super::{ActionDuration as dur, Movement::*, RotationDirection as rot};
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
