use std::collections::VecDeque;

use crate::{camera::InitializeCamera, shape::InitializeShape, text::InitializeFont};

/// The set of initialization commands to load or create assets in the initialization phase.
///
/// These are added to the [`InitializerQueue`] using [`libprim::state::State::add_initializer`]
/// and run after basic engine setup, but before setup systems are invoked.
pub enum InitializeCommand {
    /// Used to load a font into the [`libprim::text::FontRegistry`]
    InitializeFont(InitializeFont),
    /// Used to load a new shape into the [`libprim::shape_registry::ShapeRegistry`]
    InitializeShape(InitializeShape),
    /// Used to setup a non-default camera.
    InitializeCamera(InitializeCamera),
}

#[derive(Default)]
pub(crate) struct InitializerQueue {
    pub queue: VecDeque<InitializeCommand>,
}

impl InitializerQueue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
