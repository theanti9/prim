use std::{collections::HashSet, hash::BuildHasherDefault};

use hashers::fx_hash::FxHasher;
pub use winit::event::{MouseButton, VirtualKeyCode};

use crate::util::FxHashSet;

/// Stores the keyboard state at the start of each frame.
///
/// Before the world updates are run, input events are collected and pushed into a [`Keyboard`] instance,
/// which is made available as a world resource to all systems.
#[derive(Debug, Clone)]
pub struct Keyboard {
    just_pressed: FxHashSet<VirtualKeyCode>,
    currently_pressed: FxHashSet<VirtualKeyCode>,
    just_released: FxHashSet<VirtualKeyCode>,
}

impl Default for Keyboard {
    fn default() -> Self {
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
}

impl Keyboard {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears `just_*` state before processing the next set of inputs.
    pub(crate) fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Called when a key is first pressed.
    ///
    /// Persisted for one frame.
    pub(crate) fn pressed(&mut self, key: VirtualKeyCode) {
        self.just_pressed.insert(key);
        self.currently_pressed.insert(key);
    }

    /// Called when a key is released.
    ///
    /// Persisted for one frame.
    pub(crate) fn released(&mut self, key: VirtualKeyCode) {
        self.currently_pressed.remove(&key);
        self.just_released.insert(key);
    }

    /// Returns true if the given key is currently down.
    #[must_use]
    pub fn is_down(&self, key: &VirtualKeyCode) -> bool {
        self.currently_pressed.contains(key)
    }

    /// Returns true for the first frame after a key was pressed.
    #[must_use]
    pub fn just_down(&self, key: &VirtualKeyCode) -> bool {
        self.just_pressed.contains(key)
    }

    /// Returns true for the first frame after a key was released.
    #[must_use]
    pub fn just_up(&self, key: &VirtualKeyCode) -> bool {
        self.just_released.contains(key)
    }

    /// Returns the set of keys current down.
    #[inline(always)]
    #[must_use]
    pub fn currently_pressed(&self) -> &FxHashSet<VirtualKeyCode> {
        &self.currently_pressed
    }
}

/// Stores the mouse state at the start of each frame.
///
/// Before the world updates are run, input events are collected and pushed into a [`Mouse`] instance,
/// which is made available as a world resource to all systems.
#[derive(Debug, Clone)]
pub struct Mouse {
    just_pressed: FxHashSet<MouseButton>,
    currently_pressed: FxHashSet<MouseButton>,
    just_released: FxHashSet<MouseButton>,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            just_pressed: HashSet::with_capacity_and_hasher(
                4,
                BuildHasherDefault::<FxHasher>::default(),
            ),
            currently_pressed: HashSet::with_capacity_and_hasher(
                4,
                BuildHasherDefault::<FxHasher>::default(),
            ),
            just_released: HashSet::with_capacity_and_hasher(
                4,
                BuildHasherDefault::<FxHasher>::default(),
            ),
        }
    }
}

impl Mouse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears `just_*` state before processing the next set of inputs.
    pub(crate) fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Called when a mouse button is first pressed.
    ///
    /// Persisted for one frame.
    pub(crate) fn pressed(&mut self, key: MouseButton) {
        self.just_pressed.insert(key);
        self.currently_pressed.insert(key);
    }

    /// Called when a mouse button is released.
    ///
    /// Persisted for one frame.
    pub(crate) fn released(&mut self, key: MouseButton) {
        match key {
            MouseButton::Left | MouseButton::Right | MouseButton::Middle => {
                self.currently_pressed.remove(&key);
                self.just_released.insert(key);
            }
            MouseButton::Other(_) => {
                // Release events don't necessarily have the same num code as the pressed events. They seem to show up as zero.
                // Treat this as all of them have been released for now.
                let to_release: FxHashSet<MouseButton> = self
                    .currently_pressed
                    .iter()
                    .filter(|&button| matches!(button, MouseButton::Other(_)))
                    .copied()
                    .collect();
                self.just_released.extend(&to_release);
                self.currently_pressed = self
                    .currently_pressed
                    .difference(&to_release)
                    .copied()
                    .collect();
            }
        }
    }

    /// Returns true if the given mouse button is currently down.
    #[must_use]
    pub fn is_down(&self, key: &MouseButton) -> bool {
        self.currently_pressed.contains(key)
    }

    /// Returns true for the first frame after a mouse button was pressed.
    #[must_use]
    pub fn just_down(&self, key: &MouseButton) -> bool {
        self.just_pressed.contains(key)
    }

    /// Returns true for the first frame after a mouse button was released.
    ///
    /// Buttons other than Left, Right, and Middle will all be marked as released at the same time,
    /// as the incoming release event does not contain equivalent codes to the pressed event.
    #[must_use]
    pub fn just_up(&self, key: &MouseButton) -> bool {
        self.just_released.contains(key)
    }

    /// Returns the set of Mouse Buttons current down.
    #[inline(always)]
    #[must_use]
    pub fn currently_pressed(&self) -> &FxHashSet<MouseButton> {
        &self.currently_pressed
    }
}

#[cfg(test)]
mod tests {
    use winit::event::MouseButton;

    use super::Mouse;

    #[test]
    fn test_other_mouse_buttons() {
        let mut mouse = Mouse::new();

        mouse.pressed(MouseButton::Other(64));

        assert!(mouse.is_down(&MouseButton::Other(64)));
        assert!(!mouse.is_down(&MouseButton::Other(63)));
        assert!(mouse.currently_pressed().iter().next().is_some());

        assert!(mouse.just_down(&MouseButton::Other(64)));

        mouse.update();
        assert!(!mouse.just_down(&MouseButton::Other(64)));
        assert!(mouse.is_down(&MouseButton::Other(64)));

        mouse.released(MouseButton::Other(0));
        assert!(mouse.just_up(&MouseButton::Other(64)));
        assert!(!mouse.is_down(&MouseButton::Other(64)));
        assert!(mouse.currently_pressed().iter().next().is_none());
    }
}
