use bevy_ecs::{schedule::{SystemSet, ShouldRun}, system::{Commands, ResMut}};
use glam::{Vec2, Vec4};
use libprim::{
    instance::{Instance2D, InstanceBundle},
    run,
    window::PrimWindowOptions,
};

pub struct Spawned;

pub struct HasRunMarker<T>(bool, T)
where
    T: Send + Sync + 'static;

fn run_only_once<T>(mut marker: ResMut<HasRunMarker<T>>) -> ShouldRun
where
    T: Send + Sync + 'static,
{
    if !marker.0 {
        marker.0 = true;
        return ShouldRun::Yes;
    }
    ShouldRun::No
}

pub fn show_input() {
    run(PrimWindowOptions::default(), |state| {
        {
            let world = state.borrow_world();
            world.insert_resource(HasRunMarker(false, Spawned));
        }
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "pre_update",
            SystemSet::new()
                .with_run_criteria(run_only_once::<Spawned>)
                .with_system(spawn_world),
        );
    });
}

pub fn spawn_world(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(InstanceBundle::new(Instance2D {
            position: Vec2::new(-150.0, -150.0),
            scale: Vec2::splat(250.0),
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            shape: 1,
            emitter_occluder: libprim::instance::EmitterOccluder::Emitter,
            ..Default::default()
        }));
    commands
        .spawn()
        .insert_bundle(InstanceBundle::new(Instance2D {
            position: Vec2::new(150.0, 150.0),
            scale: Vec2::splat(25.0),
            color: Vec4::new(1.0, 0.5, 1.0, 1.0),
            shape: 1,
            emitter_occluder: libprim::instance::EmitterOccluder::Emitter,
            ..Default::default()
        }));
}

fn main() {
    show_input();
}
