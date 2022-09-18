use bevy_ecs::{
    prelude::Component,
    query::With,
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut},
};
use glam::{Vec2, Vec4};
use libprim::{instance::Instance2D, run, time::Time};
use log::error;
use rand::{thread_rng, Rng};

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
                .insert(Instance2D {
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
                })
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

fn main() {
    spinner_test();
    // movement_test();
}
