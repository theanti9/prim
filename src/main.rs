use bevy_ecs::{
    prelude::Component,
    query::With,
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut, Spawn},
};
use glam::{Vec2, Vec4};
use libprim::{
    input::Keyboard,
    instance::{Instance2D, InstanceBundle},
    run,
    shape_registry::ShapeRegistry,
    state::RenderState,
    time::Time,
};
use log::error;
use rand::{thread_rng, Rng};
use winit::event::VirtualKeyCode;

const NUM_INSTANCES_PER_ROW: u32 = 100;

#[derive(Component)]
pub struct Spinner;

#[derive(Component)]
pub struct SpinMultiplier(f32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    Starting,
    Playing,
}

fn spinner_test() {
    pollster::block_on(run(|state| {
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
    }));
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
    error!("Spawner");
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
                }))
                .insert(SpinMultiplier(rng.gen_range(0.2..2.0)));
        }
    }
}

fn spin(mut spinners: Query<(&mut Instance2D, &SpinMultiplier), With<Spinner>>, time: Res<Time>) {
    for (mut spinner, multiplier) in &mut spinners {
        spinner.rotation += multiplier.0 * time.delta_seconds();
    }
}
// pub struct Player {
//     instances: Vec<Instance2D>,
//     movement_controller: MovementController,
// }

// impl Player {
//     pub fn new() -> Self {
//         let mut inst = Instance2D::new();
//         inst.shape = 1;
//         inst.scale = Vec2::splat(150.0);
//         inst.color = Vec4::ONE;
//         Self {
//             instances: Vec::from([inst]),
//             movement_controller: MovementController::new(500.0, Vec2::ZERO),
//         }
//     }
// }

// impl Component for Player {
//     fn update(&mut self, time: &Time, state: &State) {
//         self.movement_controller.update(time, state);
//         self.instances[0].position = self.movement_controller.position;
//     }

//     fn get_renderables(&self) -> &Vec<Instance2D> {
//         &self.instances
//     }
// }

// fn movement_test() {
//     pollster::block_on(run(|state| {
//         state.spawn(|obj| {
//             obj.add_component(Player::new());
//         })
//     }));
// }

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(f32);

fn move_player(
    input: Res<Keyboard>,
    time: Res<Time>,
    mut player_query: Query<(&mut Instance2D, &MoveSpeed), With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if input.is_down(&VirtualKeyCode::Right) {
        direction += Vec2::X;
    }

    if input.is_down(&VirtualKeyCode::Left) {
        direction += Vec2::NEG_X;
    }

    if let Ok((mut player_inst, speed)) = player_query.get_single_mut() {
        player_inst.position += speed.0 * time.delta_seconds() * direction;
    }
}

fn spawn_world(
    mut commands: Commands,
    mut shape_registry: ResMut<ShapeRegistry>,
    render_state: Res<RenderState>,
) {
    let house_id = shape_registry.register_shape(
        "House".to_string(),
        Vec::from([
            Vec2::new(-0.5, 0.0),
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, -0.5),
            Vec2::new(0.5, 0.0),
            Vec2::new(0.25, 0.0),
            Vec2::new(0.25, 0.5),
            Vec2::new(-0.25, 0.5),
            Vec2::new(-0.25, 0.0),
        ]),
        Vec::from([0, 1, 2, 0, 2, 3, 6, 7, 5, 5, 7, 4]),
        &render_state.device,
    );

    let rocket_id = shape_registry.register_shape(
        "Rocket".to_string(),
        Vec::from([
            Vec2::new(0.0, 0.5),
            Vec2::new(-0.5, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(0.25, 0.0),
            Vec2::new(-0.25, 0.0),
            Vec2::new(-0.25, -0.5),
            Vec2::new(0.25, -0.5),
        ]),
        Vec::from([0, 1, 2, 3, 4, 5, 3, 5, 6]),
        &render_state.device,
    );

    commands
        .spawn()
        .insert_bundle(InstanceBundle::new(Instance2D {
            position: Vec2::new(0.0, -475.0),
            rotation: 0.0,
            scale: Vec2::splat(50.0),
            color: Vec4::ONE,
            shape: 1,
        }))
        .insert(Player)
        .insert(MoveSpeed(345.0))
        .insert(TimeSinceFired(0.0));

    for i in -3..3 {
        commands
            .spawn()
            .insert_bundle(InstanceBundle::new(Instance2D {
                position: Vec2::new(i as f32 * 150.0, -300.0),
                rotation: 0.0,
                scale: Vec2::new(100.0, 50.0),
                color: Vec4::new(0.7, 0.7, 0.7, 1.0),
                shape: house_id,
            }));
    }
}

pub struct Spawned;

#[derive(Component)]
pub struct TimeSinceFired(f32);

pub fn fire(input: Res<Keyboard>, mut delay: Query<&mut TimeSinceFired, With<Player>>, time: Res<Time>) {
    if let Ok(mut fire_delay) = delay.get_single_mut() {
        fire_delay.0 += time.delta_seconds();
        if fire_delay.0 < 0.5 {
            return;
        }

        
    }
}

pub fn space_invader() {
    pollster::block_on(run(|state| {
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
        schedule.add_system_to_stage("update", move_player);
    }));
}

fn main() {
    //spinner_test();
    // movement_test();
    space_invader();
}
