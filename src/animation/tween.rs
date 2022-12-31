use bevy_ecs::{
    prelude::{Component, Entity},
    query::With,
    schedule::SystemSet,
    system::{Commands, Query, Res},
};
use glam::{Vec2, Vec4};

use crate::{instance::Instance2D, time::Time};

/// Defines a tween between two states over the specified duration.
#[derive(Debug)]
pub struct FromTo<T>
where
    T: Lerp + Copy + Clone,
{
    /// The state to start from.
    pub from: T,
    /// The state to end at.
    pub to: T,
    /// The duration in seconds it takes to get between `from` and `to`.
    pub duration: f32,
}

impl<T> FromTo<T>
where
    T: Lerp + Copy + Clone,
{
    /// Create a new Tween.
    pub fn new(from: T, to: T, duration: f32) -> Self {
        Self { from, to, duration }
    }

    /// Returns the entity tweened between `from` and `to` by `time` seconds.
    pub fn lerp(&self, time: f32) -> T {
        self.from
            .tween_lerp(self.to, (time / self.duration).clamp(0.0, 1.0))
    }

    /// Checks if the tween has completed based on the total time.
    pub fn is_complete(&self, time: f32) -> bool {
        time >= self.duration
    }
}

/// Defines the attributes that can be Tweened.
#[derive(Debug, Component)]
pub enum Tween {
    /// A position Tween, moving the instance between two world positions over the specified time.
    /// 
    /// Position values are absolute world positions, not relative positions.
    Position(FromTo<Vec2>),
    /// A rotation Tween, rotating the instance between two radian values over the specified time.
    ///
    /// Radian values are absolute, not relative.
    Rotation(FromTo<f32>),
    /// A scale Tween, resizing the object between two scales over the specified time.
    Scale(FromTo<Vec2>),
    /// A color Tween, shifting the color of the instance between two colors over the specified
    /// time.
    Color(FromTo<Vec4>),
}

impl Tween {
    /// Creates a new position tween
    ///
    /// Positions are absolute world positions, not relative positions.
    #[must_use]
    pub fn tween_position(from: Vec2, to: Vec2, duration: f32) -> Self {
        Self::Position(FromTo::new(from, to, duration))
    }

    /// Creates a new rotation tween.
    ///
    /// Rotation values are absolute rotations, not relative.
    #[must_use]
    pub fn tween_rotation(from: f32, to: f32, duration: f32) -> Self {
        Self::Rotation(FromTo::new(from, to, duration))
    }

    /// Creates a new scale tween.
    ///
    /// Scale values are absolute scale values, not relative.
    #[must_use]
    pub fn tween_scale(from: Vec2, to: Vec2, duration: f32) -> Self {
        Self::Scale(FromTo::new(from, to, duration))
    }

    /// Creates a new color tween.
    #[must_use]
    pub fn tween_color(from: Vec4, to: Vec4, duration: f32) -> Self {
        Self::Color(FromTo::new(from, to, duration))
    }
}

/// Contains the current amount of time attached tweens have been playing.
#[derive(Debug, Component, Default)]
pub struct TweenState {
    /// The time in Seconds since the tween started.
    pub time: f32,
}

/// Contains a set of tweens to execute at the same time.
#[derive(Debug, Component)]
pub struct Tweens(pub Vec<Tween>);

/// Indicates that the current [`Entity`] has executing tweens.
///
/// Removing this [`Component`] will pause any active Tweens.
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

/// Defines a type which can be linearly ineterpolated between two values.
pub trait Lerp {
    /// Lerp between `self` and another instance by the specified percentage.
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

/// A [`SystemSet`] for executing Tweens.
///
/// This system set should be added to the [`libprim::state::CoreStages::Update`] stage to behave
/// properly 
pub fn tween_system_set() -> SystemSet {
    SystemSet::new().with_system(tween)
}
