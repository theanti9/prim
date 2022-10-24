use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    schedule::SystemSet,
    system::{Commands, Query, Res},
};
use glam::{Vec2, Vec4};

use crate::{instance::Instance2D, time::Time};

#[derive(Debug)]
pub struct FromTo<T>
where
    T: Lerp + Copy + Clone,
{
    pub from: T,
    pub to: T,
    pub duration: f32,
}

impl<T> FromTo<T>
where
    T: Lerp + Copy + Clone,
{
    pub fn new(from: T, to: T, duration: f32) -> Self {
        Self { from, to, duration }
    }

    pub fn lerp(&self, time: f32) -> T {
        self.from
            .tween_lerp(self.to, (time / self.duration).clamp(0.0, 1.0))
    }

    pub fn is_complete(&self, time: f32) -> bool {
        time >= self.duration
    }
}

#[derive(Debug, Component)]
pub enum Tween {
    Position(FromTo<Vec2>),
    Rotation(FromTo<f32>),
    Scale(FromTo<Vec2>),
    Color(FromTo<Vec4>),
}

impl Tween {
    #[must_use]
    pub fn tween_position(from: Vec2, to: Vec2, duration: f32) -> Self {
        Self::Position(FromTo::new(from, to, duration))
    }

    #[must_use]
    pub fn tween_rotation(from: f32, to: f32, duration: f32) -> Self {
        Self::Rotation(FromTo::new(from, to, duration))
    }

    #[must_use]
    pub fn tween_scale(from: Vec2, to: Vec2, duration: f32) -> Self {
        Self::Scale(FromTo::new(from, to, duration))
    }

    #[must_use]
    pub fn tween_color(from: Vec4, to: Vec4, duration: f32) -> Self {
        Self::Color(FromTo::new(from, to, duration))
    }
}

#[derive(Debug, Component, Default)]
pub struct TweenState {
    pub time: f32,
}

#[derive(Debug, Component)]
pub struct Tweens(pub Vec<Tween>);

#[derive(Debug, Component)]
pub struct Tweening;

fn tween(
    mut tweens: Query<(Entity, &mut Instance2D, &mut TweenState, &Tweens), With<Tweening>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut instance, mut tween_state, tweens) in tweens.iter_mut() {
        tween_state.time += time.delta_seconds();
        let mut done = true;
        for tween in &tweens.0 {
            match tween {
                Tween::Position(from_to_position) => {
                    instance.position = from_to_position.lerp(tween_state.time);
                    done &= from_to_position.is_complete(tween_state.time);
                }
                Tween::Rotation(from_to_rotation) => {
                    instance.rotation = from_to_rotation.lerp(tween_state.time);
                    done &= from_to_rotation.is_complete(tween_state.time);
                }
                Tween::Scale(from_to_scale) => {
                    instance.scale = from_to_scale.lerp(tween_state.time);
                    done &= from_to_scale.is_complete(tween_state.time);
                }
                Tween::Color(from_to_color) => {
                    instance.color = from_to_color.lerp(tween_state.time);
                    done &= from_to_color.is_complete(tween_state.time);
                }
            }
        }

        if done {
            commands
                .entity(entity)
                .remove::<Tweening>()
                .remove::<Tweens>()
                .remove::<TweenState>();
        }
    }
}

pub trait Lerp {
    #[must_use]
    fn tween_lerp(&self, rhs: Self, pct: f32) -> Self;
}

impl Lerp for Vec2 {
    fn tween_lerp(&self, rhs: Self, pct: f32) -> Self {
        self.lerp(rhs, pct)
    }
}

impl Lerp for f32 {
    fn tween_lerp(&self, rhs: Self, pct: f32) -> Self {
        self * (1.0 - pct) + (rhs * pct)
    }
}

impl Lerp for Vec4 {
    fn tween_lerp(&self, rhs: Self, pct: f32) -> Self {
        self.lerp(rhs, pct)
    }
}

pub fn tween_system_set() -> SystemSet {
    SystemSet::new().with_system(tween)
}
