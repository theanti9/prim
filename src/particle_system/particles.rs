use bevy_ecs::prelude::Component;
use glam::Vec2;


#[derive(Component)]
pub struct ParticleSystem {
    pub shape_id: u32,
    pub spawn_rate: f32,
    pub max_particles: usize,
    pub emitter_direction: Vec2,
    pub emitter_shape: f32
}

