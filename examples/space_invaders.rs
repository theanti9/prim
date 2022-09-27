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
    particle_system::{
        components::{
            EmitterPosition, ParticleBurst, ParticleSystem, ParticleSystemBundle, Playing,
            TimeScale,
        },
        systems::system_set,
        values::JitteredValue,
    },
    run,
    shape_registry::ShapeRegistry,
    state::{FpsDisplayBundle, InitializeCommand, RenderState},
    text::{InitializeFont, TextSection},
    time::Time,
};
use wgpu_text::section::{OwnedText, Section, Text};
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
                commands
                    .spawn()
                    .insert_bundle(ParticleSystemBundle {
                        particle_system: ParticleSystem {
                            max_particles: 10,
                            shape_id: 2,
                            spawn_rate_per_second: 0.0.into(),
                            emitter_shape: 45.0_f32.to_radians(),
                            emitter_angle: 270.0_f32.to_radians(),
                            initial_velocity: JitteredValue::jittered(150.0, -50.0..50.0),
                            acceleration: 0.0.into(),
                            lifetime: JitteredValue::jittered(0.4, -0.2..0.2),
                            color: Vec4::new(0.6, 0.6, 0.6, 0.6).into(),
                            scale: 10.0.into(),
                            looping: false,
                            system_duration_seconds: 2.0,
                            max_distance: 100.0.into(),
                            bursts: vec![ParticleBurst::new(0.0, 5)],
                            use_scaled_time: false,
                            despawn_on_finish: true,
                            ..Default::default()
                        },
                        position: EmitterPosition(inst.position + Vec2::new(0.0, 50.0)),
                        ..Default::default()
                    })
                    .insert(Playing);
            }
        }
    }
}

pub fn player_fire_collision(
    collision_query: Query<(Entity, &Instance2D, &Colliding<PlayerFire>), With<PlayerFire>>,
    inst_query: Query<&Instance2D>,
    mut score: ResMut<Score>,
    mut commands: Commands,
) {
    for (entity, inst, collisions) in &collision_query {
        commands.entity(entity).despawn();
        // rocket explosion
        commands
            .spawn()
            .insert_bundle(ParticleSystemBundle {
                particle_system: ParticleSystem {
                    max_particles: 25,
                    shape_id: 2,
                    spawn_rate_per_second: 100.0.into(),
                    initial_velocity: 50.0.into(),
                    lifetime: JitteredValue::jittered(0.4, -0.2..0.1),
                    color: Vec4::new(1.0, 0.65, 0.0, 1.0).into(),
                    scale: 25.0.into(),
                    looping: false,
                    system_duration_seconds: 0.2,
                    max_distance: 50.0.into(),
                    bursts: vec![],
                    despawn_on_finish: true,
                    ..Default::default()
                },
                position: EmitterPosition(inst.position),
                ..Default::default()
            })
            .insert(Playing);

        for collision in &collisions.0 {
            // enemy splat
            if let Ok(enemy_inst) = inst_query.get_component::<Instance2D>(*collision) {
                let angle = enemy_inst.position.angle_between(inst.position);
                commands
                    .spawn()
                    .insert_bundle(ParticleSystemBundle {
                        particle_system: ParticleSystem {
                            max_particles: 35,
                            shape_id: 2,
                            spawn_rate_per_second: 100.0.into(),
                            initial_velocity: 300.0.into(),
                            emitter_shape: 45.0_f32.to_radians(),
                            emitter_angle: angle + 90.0_f32.to_radians(),
                            lifetime: JitteredValue::jittered(0.35, -0.2..0.1),
                            color: Vec4::new(0.25, 0.9, 0.6, 1.0).into(),
                            scale: 10.0.into(),
                            looping: false,
                            system_duration_seconds: 0.2,
                            max_distance: 100.0.into(),
                            bursts: vec![],
                            despawn_on_finish: true,
                            ..Default::default()
                        },
                        position: EmitterPosition(inst.position),
                        ..Default::default()
                    })
                    .insert(Playing);
            }
            commands.entity(*collision).despawn();
            score.0 += 10;
        }
    }
}

#[derive(Component)]
pub struct ScoreDisplay;

pub fn score_display(mut query: Query<&mut TextSection, With<ScoreDisplay>>, score: Res<Score>) {
    if score.is_changed() {
        if let Ok(mut text_section) = query.get_single_mut() {
            text_section.section.text[0] = OwnedText::default()
                .with_text(format!("{}", score.0))
                .with_color(Vec4::ONE)
                .with_scale(32.0);
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

    commands.spawn().insert_bundle(FpsDisplayBundle::default());
    commands.spawn().insert(ScoreDisplay).insert(TextSection {
        font_id: 0,
        section: Section::default()
            .with_text(vec![Text::default()
                .with_text("0")
                .with_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_scale(32.0)])
            .with_screen_position((render_state.config.width as f32 / 2.0, 0.0))
            .to_owned(),
    });
}

#[derive(Default)]
pub struct Score(u32);

pub fn space_invader() {
    run(|state| {
        state.add_initializer(InitializeCommand::InitializeFont(InitializeFont::new(
            "RobotoMono".to_string(),
            include_bytes!("../assets/fonts/RobotoMono-Regular.ttf"),
        )));
        {
            let world = state.borrow_world();
            world.insert_resource(HasRunMarker(false, Spawned));
            world.insert_resource(HashGrid { size: 100 });
            world.init_resource::<Option<TimeScale>>();
            world.insert_resource(Score::default());
        }
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "pre_update",
            SystemSet::new()
                .with_run_criteria(run_only_once::<Spawned>)
                .with_system(spawn_world),
        );
        schedule.add_system_set_to_stage("update", system_set());

        schedule.add_system_set_to_stage("pre_update", base_collision_detection());
        schedule.add_system_set_to_stage("pre_update", collision_system_set::<Player>());
        schedule.add_system_set_to_stage("pre_update", collision_system_set::<PlayerFire>());

        schedule.add_system_to_stage("update", move_player);
        schedule.add_system_to_stage("update", fire);
        schedule.add_system_to_stage("update", player_fire_movement);
        schedule.add_system_to_stage("update", player_fire_collision);
        schedule.add_system_to_stage("update", score_display);
    });
}

fn main() {
    space_invader();
}
