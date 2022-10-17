use std::collections::VecDeque;

use crate::{shape::InitializeShape, text::InitializeFont};

pub enum InitializeCommand {
    InitializeFont(InitializeFont),
    InitializeShape(InitializeShape),
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
