#[derive(Debug, Clone, Copy)]
pub struct Time {
    start: instant::Instant,
    current_instant: instant::Instant,
    previous_instant: instant::Instant,
    delta_seconds: f32,
}

impl Time {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.previous_instant = self.current_instant;
        self.current_instant = instant::Instant::now();
        self.delta_seconds = self
            .current_instant
            .duration_since(self.previous_instant)
            .as_secs_f32();
    }

    #[inline(always)]
    #[must_use]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

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
