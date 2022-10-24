use bevy_ecs::{
    prelude::{Bundle, Component},
    query::With,
    schedule::SystemSet,
    system::{Query, Res},
};

use crate::{instance::Instance2D, time::Time};

#[derive(Debug)]
pub struct TimePoint {
    pub shape_id: u32,
    pub duration: f32,
}

#[derive(Debug, Component)]
pub struct Animation {
    pub time_points: Vec<TimePoint>,
    duration: f32,
    pub looping: bool,
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
pub struct AnimationState {
    pub current_index: usize,
    pub current_time: f32,
}

#[derive(Debug, Component)]
pub struct Animating;

#[derive(Debug, Bundle)]
pub struct AnimationBundle {
    animation: Animation,
    animation_state: AnimationState,
    animating: Animating,
}

impl AnimationBundle {
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

pub fn animation_system_set() -> SystemSet {
    SystemSet::new().with_system(update_animations)
}
