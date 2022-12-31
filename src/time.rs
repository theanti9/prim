/// The [`Time`] struct is stored as a Resource in the bevy world,
/// updated at the start of each frame, tracking runtime duration and time between frames.
///
/// This can be referenced from any system to make game logic frame-rate independent.
///
/// ## Example
///
/// ```
/// # use libprim::time::Time;
/// # use bevy_ecs::system::Res;
/// fn my_system(time: Res<Time>) {
///     log::info!("Time since last frame: {}", time.delta_seconds());
///     log::info!("Total runtime: {}", time.total_seconds());
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// The [`Instant`] the engine initialized.
    start: instant::Instant,
    /// The [`Instant`] the beginning of the current frame began.
    current_instant: instant::Instant,
    /// The [`Instant`] the previous frame began.
    previous_instant: instant::Instant,
    /// The number of seconds as a float between the current frame and the previous frame.
    delta_seconds: f32,
}

impl Time {
    /// Creates a new [`Time`] with all [`Instant`] fields being the current time.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Called once per frame to rotate `previous_instant` and `current_instant`.
    ///
    /// Precomputes `self.delta_seconds` so that it can be referenced many times without
    /// wasted cycles.
    #[inline(always)]
    pub(crate) fn update(&mut self) {
        self.previous_instant = self.current_instant;
        self.current_instant = instant::Instant::now();
        self.delta_seconds = self
            .current_instant
            .duration_since(self.previous_instant)
            .as_secs_f32();
    }

    /// Get the amount of seconds between the previos frame and this frame.
    #[inline(always)]
    #[must_use]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Get the total amount of seconds that the engine has been running.
    #[must_use]
    pub fn total_seconds(&self) -> f32 {
        self.current_instant
            .duration_since(self.start)
            .as_secs_f32()
    }
}

impl Default for Time {
    fn default() -> Self {
        let now = instant::Instant::now();
        Self {
            start: now,
            current_instant: now,
            previous_instant: now,
            delta_seconds: Default::default(),
        }
    }
}
