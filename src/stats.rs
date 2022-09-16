use cfg_if::cfg_if;
#[cfg(feature = "stats")]
use log::error;

#[allow(dead_code)]
pub struct CoreStats {
    frame_start: instant::Instant,
    update_start: instant::Instant,
    render_start: instant::Instant,
    draw_calls: usize,
    frames: u32,
    total_frame_time: f32,
    total_update_time: f32,
    total_render_time: f32,
    last_log: instant::Instant,
}

impl CoreStats {
    pub fn new() -> Self {
        let now = instant::Instant::now();
        Self {
            frame_start: now,
            update_start: now,
            render_start: now,
            draw_calls: 0,
            frames: 0,
            total_frame_time: 0.0,
            total_update_time: 0.0,
            total_render_time: 0.0,
            last_log: now,
        }
    }

    #[inline(always)]
    pub fn frame_start(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.frame_start = instant::Instant::now();
            }
        }
    }

    #[inline(always)]
    pub fn update_start(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.update_start = instant::Instant::now();
            }
        }
    }

    #[inline(always)]
    pub fn render_start(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.render_start = instant::Instant::now();
            }
        }
    }

    #[inline(always)]
    pub fn draw_call(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.draw_calls += 1;
            }
        }
    }

    #[inline(always)]
    pub fn update_end(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.total_update_time += instant::Instant::now().duration_since(self.update_start).as_secs_f32();
            }
        }
    }

    #[inline(always)]
    pub fn render_end(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                self.total_render_time += instant::Instant::now().duration_since(self.render_start).as_secs_f32();
            }
        }
    }

    #[inline(always)]
    pub fn frame_end(&mut self) {
        cfg_if! {
            if #[cfg(feature = "stats")] {
                let now = instant::Instant::now();
                self.frames += 1;
                self.total_frame_time += now
                    .duration_since(self.frame_start)
                    .as_secs_f32();

                if now.duration_since(self.last_log).as_secs_f32() >= 5.0 {
                    error!("-------------");
                    error!("Avg FPS: {:.3}/s", self.frames as f32 / self.total_frame_time);
                    error!("Avg draws: {}", self.draw_calls as f32 / self.frames as f32);
                    error!("Avg update: {:.3}ms", self.total_update_time / self.frames as f32 * 1000.0);
                    error!("Avg render: {:.3}ms", self.total_render_time / self.frames as f32 * 1000.0);
                    error!("-------------");
                    self.frames = 0;
                    self.total_frame_time = 0.0;
                    self.total_update_time = 0.0;
                    self.total_render_time = 0.0;
                    self.draw_calls = 0;
                    self.last_log = now;
                }

            }
        }
    }
}
