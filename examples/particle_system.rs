use bevy_ecs::{
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut},
};
use glam::Vec4;
use libprim::{
    particle_system::{
        components::{
            ParticleBurst, ParticleCount, ParticleSystem, ParticleSystemBundle, Playing, TimeScale,
        },
        systems::system_set,
        values::{ColorOverTime, ColorPoint, Gradient, JitteredValue, SinWave, ValueOverTime},
    },
    run,
    time::Time,
};

use log::error;

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

pub struct Spawned;

fn spawn_world(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(ParticleSystemBundle {
            particle_system: ParticleSystem {
                max_particles: 50000,
                shape_id: 1,
                spawn_rate_per_second: 1000.0.into(),
                initial_velocity: JitteredValue::jittered(50.0, -10.0..10.0),
                acceleration: ValueOverTime::Sin(SinWave {
                    amplitude: 150.0,
                    period: 5.0,
                    ..SinWave::default()
                }),
                lifetime: JitteredValue::jittered(10.0, -2.0..2.0),
                color: ColorOverTime::Gradient(Gradient::new(vec![
                    ColorPoint::new(Vec4::new(0.5, 0.0, 0.5, 1.0), 0.0),
                    ColorPoint::new(Vec4::new(1.0, 0.0, 0.0, 1.0), 0.5),
                    ColorPoint::new(Vec4::new(0.0, 0.0, 1.0, 0.0), 1.0),
                ])),
                scale: 20.0.into(),
                looping: true,
                system_duration_seconds: 10.0,
                // max_distance: Some(700.0),
                bursts: vec![
                    ParticleBurst::new(0.0, 1000),
                    ParticleBurst::new(2.0, 1000),
                    ParticleBurst::new(4.0, 1000),
                    ParticleBurst::new(6.0, 1000),
                    ParticleBurst::new(8.0, 1000),
                ],
                ..ParticleSystem::default()
            },
            ..ParticleSystemBundle::default()
        })
        .insert(Playing);
}

pub struct TimeSinceLog(f32);

pub fn particle_counter(
    particle_system_query: Query<&ParticleCount>,
    mut last_log: ResMut<TimeSinceLog>,
    time: Res<Time>,
) {
    last_log.0 += time.delta_seconds();
    if last_log.0 > 10.0 {
        last_log.0 = 0.0;
        for particle_count in &particle_system_query {
            error!("Alive particles: {}", particle_count.0);
        }
    }
}

fn main() {
    run(|state| {
        let world = state.borrow_world();
        world.init_resource::<Option<TimeScale>>();
        world.insert_resource(HasRunMarker(false, Spawned));
        world.insert_resource(TimeSinceLog(0.0));

        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "pre_update",
            SystemSet::new()
                .with_run_criteria(run_only_once::<Spawned>)
                .with_system(spawn_world),
        );
        schedule.add_system_set_to_stage("update", system_set());
        schedule.add_system_to_stage("update", particle_counter);
    });
}
