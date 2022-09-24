use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut},
};
use glam::{Vec2, Vec4};
use libprim::{
    collision::{
        base_collision_detection, collision_system_set, Collidable, Collider, CollidesWith,
        Colliding, HashGrid,
    },
    input::Keyboard,
    instance::{Instance2D, InstanceBundle},
    run,
    shape_registry::ShapeRegistry,
    state::RenderState,
    time::Time,
};
use log::error;
use winit::event::VirtualKeyCode;

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
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(f32);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct PlayerFire;

#[derive(Component)]
pub struct EnemyFire;

pub struct Spawned;

#[derive(Component)]
pub struct TimeSinceFired(f32);

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

pub fn fire(
    input: Res<Keyboard>,
    mut delay: Query<(&mut TimeSinceFired, &Instance2D), With<Player>>,
    time: Res<Time>,
    shape_registry: Res<ShapeRegistry>,
    mut commands: Commands,
) {
    if let Ok((mut fire_delay, inst)) = delay.get_single_mut() {
        fire_delay.0 += time.delta_seconds();
        if fire_delay.0 < 0.2 {
            return;
        }
        if input.is_down(&VirtualKeyCode::Space) {
            error!("Fire from {:?}", inst.position);
            fire_delay.0 = 0.0;
            if let Some(rocket_id) = shape_registry.get_id("Rocket") {
                commands
                    .spawn()
                    .insert_bundle(InstanceBundle::new(Instance2D {
                        position: inst.position + Vec2::new(0.0, 50.0),
                        rotation: 0.0,
                        scale: Vec2::splat(25.0),
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        shape: rocket_id,
                    }))
                    .insert(PlayerFire)
                    .insert(Collidable)
                    .insert(Collider::<PlayerFire>::new())
                    .insert(CollidesWith::<Enemy>::new());
            }
        }
    }
}

pub fn player_fire_collision(
    collision_query: Query<(Entity, &Colliding<PlayerFire>), With<PlayerFire>>,
    mut commands: Commands,
) {
    for (entity, collisions) in &collision_query {
        commands.entity(entity).despawn();
        for collision in &collisions.0 {
            commands.entity(*collision).despawn();
        }
    }
}

pub fn player_fire_movement(
    mut rockets: Query<(Entity, &mut Instance2D), With<PlayerFire>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut rocket_inst) in &mut rockets {
        rocket_inst.position += time.delta_seconds() * Vec2::new(0.0, 200.0);
        //error!("Rocket position: {:?}", rocket_inst.position);
        if rocket_inst.position.y >= 2000.0 {
            commands.entity(entity).despawn();
        }
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

    shape_registry.register_shape(
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
            position: Vec2::new(249.0, -475.0),
            rotation: 0.0,
            scale: Vec2::splat(50.0),
            color: Vec4::ONE,
            shape: 1,
        }))
        .insert(Player)
        .insert(MoveSpeed(345.0))
        .insert(TimeSinceFired(0.0))
        .insert(Collidable)
        .insert(Collider::<Player>::new());

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

    let base_x = -1000.0;
    let space = 50.0;
    let base_y = 1000.0;

    let enemies_per_row = 40;
    let rows = 10;
    for y in 0..rows {
        for x in 0..enemies_per_row {
            commands
                .spawn()
                .insert_bundle(InstanceBundle::new(Instance2D {
                    position: Vec2::new(
                        base_x + space as f32 * x as f32,
                        base_y - space as f32 * y as f32,
                    ),
                    rotation: 180.0_f32.to_radians(),
                    scale: Vec2::splat(35.0),
                    color: Vec4::new(0.25, 0.9, 0.6, 1.0),
                    shape: 1,
                }))
                .insert(Enemy)
                .insert(Collidable)
                .insert(Collider::<Enemy>::new())
                .insert(CollidesWith::<PlayerFire>::new());
        }
    }
}

pub fn space_invader() {
    run(|state| {
        {
            let world = state.borrow_world();
            world.insert_resource(HasRunMarker(false, Spawned));
            world.insert_resource(HashGrid { size: 100 });
        }
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "pre_update",
            SystemSet::new()
                .with_run_criteria(run_only_once::<Spawned>)
                .with_system(spawn_world),
        );

        schedule.add_system_set_to_stage("pre_update", base_collision_detection());
        schedule.add_system_set_to_stage("pre_update", collision_system_set::<Player>());
        schedule.add_system_set_to_stage("pre_update", collision_system_set::<PlayerFire>());

        schedule.add_system_to_stage("update", move_player);
        schedule.add_system_to_stage("update", fire);
        schedule.add_system_to_stage("update", player_fire_movement);
        schedule.add_system_to_stage("update", player_fire_collision);
    });
}

fn main() {
    space_invader();
}
