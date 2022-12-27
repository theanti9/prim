#![allow(clippy::too_many_arguments)]

use bevy_ecs::{
    prelude::Component,
    query::With,
    system::{Commands, Query, Res},
};
use glam::{Vec2, Vec3, Vec4};
use libprim::{
    initialization::InitializeCommand,
    instance::{Instance2D, InstanceBundle, Outline},
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
        state.add_setup_system(spinner_spawn);
        let schedule = state.borrow_schedule();
        schedule.add_system_to_stage("update", spin);
        state.add_initializer(InitializeCommand::InitializeFont(InitializeFont {
            name: "RobotoMono".to_string(),
            bytes: include_bytes!("../assets/fonts/RobotoMono-Regular.ttf"),
        }));
    });
}

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
                    shape: u32::from((x + y) % 2 == 0),
                    outline: Some(Outline {
                        scale: 5.0,
                        color: Vec4::ZERO,
                    }),
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
