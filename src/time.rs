#[derive(Debug, Clone, Copy)]
pub struct Time {
    start: instant::Instant,
    current_instant: instant::Instant,
    previous_instant: instant::Instant,
    delta_seconds: f32,
}

impl Time {
    pub fn new() -> Self {
        let now = instant::Instant::now();
        Self {
            start: now,
            current_instant: now,
            previous_instant: now,
            delta_seconds: 0.0,
        }
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
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    pub fn total_seconds(&self) -> f32 {
        self.current_instant
            .duration_since(self.start)
            .as_secs_f32()
    }
}
