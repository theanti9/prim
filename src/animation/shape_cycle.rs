use bevy_ecs::{
    prelude::{Bundle, Component},
    query::With,
    schedule::SystemSet,
    system::{Query, Res},
};

use crate::{instance::Instance2D, time::Time};

/// Defines what shape to use and for how long that shape should remain displayed.
#[derive(Debug)]
pub struct TimePoint {
    /// The ID of the shape to display at this time point.
    pub shape_id: u32,
    /// How long the specified shape should be displayed.
    pub duration: f32,
}

/// An animation is defined by a set of ordered [`TimePoint`]s to determine when to move between
/// multiple shapes.
#[derive(Debug, Component)]
pub struct Animation {
    /// The shapes and how long to display them which make up the animation overall.
    pub time_points: Vec<TimePoint>,
    duration: f32,
    /// Whether the animation should start over when it reaches the end.
    pub looping: bool,
    /// The speed with which to move between [`TimePoint`]s.
    ///
    /// `1.0` results in the speed as defined. Values lower than `1.0` will slow down the
    /// animation, whereas values above `1.0` will speed it up.
    ///
    /// E.x. a speed of `2.0` will double the animation speed (and halve the time), while a speed
    /// of `0.5` will cut the speed in half but double the total time.
    pub speed: f32,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            time_points: Vec::default(),
            duration: 0.0,
            looping: true,
            speed: 1.0,
        }
    }
}

impl Animation {
    /// Creates a new Animation from the given parameters.
    #[must_use]
    pub fn new(time_points: Vec<TimePoint>, looping: bool, speed: f32) -> Self {
        let duration = time_points.iter().map(|t| t.duration).sum();
        Self {
            time_points,
            duration,
            looping,
            speed,
        }
    }

    fn index_for_time(&self, time: f32) -> usize {
        let mut duration = 0.0;

        if time > self.duration {
            return self.time_points.len() - 1;
        }

        for index in 0..self.time_points.len() {
            if time < self.time_points[index].duration + duration {
                return index;
            }

            duration += self.time_points[index].duration;
        }

        self.time_points.len() - 1
    }
}

#[derive(Debug, Component, Default)]
pub(crate) struct AnimationState {
    pub current_index: usize,
    pub current_time: f32,
}

/// An entity with this Marker component will play the attached [`Animation`].
///
/// Removing this [`Component`] will pause the animation.
#[derive(Debug, Component)]
pub struct Animating;

/// A bundle to include all of the components necessary for an animation to work.
#[derive(Debug, Bundle)]
pub struct AnimationBundle {
    animation: Animation,
    animation_state: AnimationState,
    animating: Animating,
}

impl AnimationBundle {
    /// Create a bundle for a given animation.
    #[must_use]
    pub fn from_animation(animation: Animation) -> Self {
        Self {
            animation,
            animation_state: AnimationState::default(),
            animating: Animating,
        }
    }
}

fn update_animations(
    mut animators: Query<(&mut Instance2D, &mut AnimationState, &Animation), With<Animating>>,
    time: Res<Time>,
) {
    for (mut instance, mut animation_state, animation) in animators.iter_mut() {
        animation_state.current_time += time.delta_seconds();
        if animation_state.current_time > animation.duration && animation.looping {
            animation_state.current_time -= animation.duration;
        }

        animation_state.current_index = animation.index_for_time(animation_state.current_time);
        instance.shape = animation.time_points[animation_state.current_index].shape_id;
    }
}

/// A [`SystemSet`] to be added to the schedule to handle playing animations.
///
/// This should be added to the [`libprim::state::CoreStages::Update`] stage to behave correctly.
pub fn animation_system_set() -> SystemSet {
    SystemSet::new().with_system(update_animations)
}
