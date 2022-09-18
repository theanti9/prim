use std::{collections::HashSet, hash::BuildHasherDefault};

use hashers::fx_hash::FxHasher;
use winit::event::VirtualKeyCode;

#[derive(Debug, Clone)]
pub struct Keyboard {
    just_pressed: HashSet<VirtualKeyCode, BuildHasherDefault<FxHasher>>,
    currently_pressed: HashSet<VirtualKeyCode, BuildHasherDefault<FxHasher>>,
    just_released: HashSet<VirtualKeyCode, BuildHasherDefault<FxHasher>>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            just_pressed: HashSet::with_capacity_and_hasher(
                10,
                BuildHasherDefault::<FxHasher>::default(),
            ),
            currently_pressed: HashSet::with_capacity_and_hasher(
                10,
                BuildHasherDefault::<FxHasher>::default(),
            ),
            just_released: HashSet::with_capacity_and_hasher(
                10,
                BuildHasherDefault::<FxHasher>::default(),
            ),
        }
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&mut self, key: VirtualKeyCode) {
        self.just_pressed.insert(key);
        self.currently_pressed.insert(key);
    }

    pub fn released(&mut self, key: VirtualKeyCode) {
        self.currently_pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub fn is_down(&self, key: &VirtualKeyCode) -> bool {
        self.currently_pressed.contains(key)
    }

    pub fn just_down(&self, key: &VirtualKeyCode) -> bool {
        self.just_pressed.contains(key)
    }

    pub fn just_up(&self, key: &VirtualKeyCode) -> bool {
        self.just_released.contains(key)
    }
}
