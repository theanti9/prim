use bevy_ecs::{
    schedule::ShouldRun,
    system::ResMut,
};

pub(crate) struct Setup;

pub(crate) struct HasRunMarker<T>(pub bool, pub T)
where
    T: Send + Sync + 'static;

pub(crate) fn run_only_once<T>(mut marker: ResMut<HasRunMarker<T>>) -> ShouldRun
where
    T: Send + Sync + 'static,
{
    if !marker.0 {
        marker.0 = true;
        return ShouldRun::Yes;
    }
    ShouldRun::No
}

