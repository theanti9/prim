use bevy_ecs::{
    prelude::Component,
    query::With,
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut},
};
use glam::{Vec2, Vec3, Vec4};
use libprim::{
    initialization::InitializeCommand,
    instance::{EmitterOccluder, Instance2D, InstanceBundle},
    run,
    state::FpsDisplayBundle,
    text::InitializeFont,
    time::Time,
    window::PrimWindowOptions,
};
use rand::{thread_rng, Rng};

const NUM_INSTANCES_PER_ROW: u32 = 100;

#[derive(Component)]
pub struct Spinner;

#[derive(Component)]
pub struct SpinMultiplier(f32);

fn spinner_test() {
    let window_options = PrimWindowOptions::default()
        .with_window_title("Prim Render Test".to_string())
        .with_vsync(true)
        .with_clear_color(Vec3::new(0.01, 0.01, 0.01));

    run(window_options, |state| {
        {
            let world = state.borrow_world();
            world.insert_resource(HasRunMarker(false, SpinSpawner));
        }
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "update",
            SystemSet::new()
                .with_system(spinner_spawn)
                .with_run_criteria(run_only_once::<SpinSpawner>),
        );
        schedule.add_system_to_stage("update", spin);
        state.add_initializer(InitializeCommand::InitializeFont(InitializeFont {
            name: "RobotoMono".to_string(),
            bytes: include_bytes!("../assets/fonts/RobotoMono-Regular.ttf"),
        }));
    });
}

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

#[derive(Component)]
pub struct SpinSpawner;

fn spinner_spawn(mut commands: Commands) {
    let mut rng = thread_rng();
    for y in 0..NUM_INSTANCES_PER_ROW {
        for x in 0..NUM_INSTANCES_PER_ROW {
            let position = Vec2::new(
                (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
                (y as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
            );
            commands
                .spawn()
                .insert(Spinner)
                .insert_bundle(InstanceBundle::new(Instance2D {
                    position,
                    rotation: 0.0,
                    scale: Vec2::splat(35.0),
                    color: Vec4::new(
                        position.x / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                        position.y / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                        0.2,
                        1.0,
                    ),
                    shape: if (x + y) % 2 == 0 { 1 } else { 0 },
                    emitter_occluder: if (x + y) % 2 == 0 {
                        EmitterOccluder::Occluder
                    } else {
                        EmitterOccluder::Emitter
                    },
                }))
                .insert(SpinMultiplier(rng.gen_range(0.2..2.0)));
        }
    }

    commands.spawn().insert_bundle(FpsDisplayBundle::new());
}

fn spin(mut spinners: Query<(&mut Instance2D, &SpinMultiplier), With<Spinner>>, time: Res<Time>) {
    for (mut spinner, multiplier) in &mut spinners {
        spinner.rotation += multiplier.0 * time.delta_seconds();
    }
}

fn main() {
    spinner_test();
}
