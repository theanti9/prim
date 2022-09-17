use glam::Vec2;
use winit::event::VirtualKeyCode;

use crate::{instance::Instance2D, object_registry::Component, state::State, time::Time};

pub struct MovementController {
    pub speed: f32,
    pub position: Vec2,
    instances: Vec<Instance2D>,
}

impl MovementController {
    pub fn new(speed: f32, position: Vec2) -> Self {
        Self {
            speed,
            position,
            instances: vec![],
        }
    }
}

impl Component for MovementController {
    fn update(&mut self, time: &Time, state: &State) {
        let mut direction = Vec2::ZERO;
        let keyboard = state.get_keyboard();
        if keyboard.is_down(&VirtualKeyCode::W) || keyboard.is_down(&VirtualKeyCode::Up) {
            direction += Vec2::Y;
        }

        if keyboard.is_down(&VirtualKeyCode::S) || keyboard.is_down(&VirtualKeyCode::Down) {
            direction += Vec2::NEG_Y;
        }

        if keyboard.is_down(&VirtualKeyCode::D) || keyboard.is_down(&VirtualKeyCode::Right) {
            direction += Vec2::X;
        }

        if keyboard.is_down(&VirtualKeyCode::A) || keyboard.is_down(&VirtualKeyCode::Left) {
            direction += Vec2::NEG_X;
        }

        if direction != Vec2::ZERO {
            self.position += self.speed * time.delta_seconds() * direction.normalize_or_zero();
        }
    }

    fn get_renderables(&self) -> &Vec<Instance2D> {
        &self.instances
    }
}
