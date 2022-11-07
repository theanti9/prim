use glam::Vec3;
use wgpu::SurfaceConfiguration;
use winit::window::Fullscreen;

#[derive(Debug)]
pub enum PrimWindowMode {
    Window,
    Fullscreen,
}

impl Default for PrimWindowMode {
    fn default() -> Self {
        Self::Window
    }
}

#[derive(Debug)]
pub struct PrimWindowOptions {
    pub window_mode: PrimWindowMode,
    pub window_size: Option<(u32, u32)>,
    pub window_title: String,
    pub window_decorations: bool,
    pub vsync: bool,
    pub clear_color: Vec3,
    pub sample_count: u32,
}

impl Default for PrimWindowOptions {
    fn default() -> Self {
        Self {
            window_mode: PrimWindowMode::default(),
            window_size: None,
            window_title: "Prim App".to_string(),
            window_decorations: true,
            vsync: false,
            clear_color: Vec3::new(0.0, 0.0, 0.0),
            sample_count: 1,
        }
    }
}

impl PrimWindowOptions {
    #[must_use]
    pub fn with_window_mode(mut self, mode: PrimWindowMode) -> Self {
        self.window_mode = mode;
        self
    }

    #[must_use]
    pub fn with_window_title(mut self, title: String) -> Self {
        self.window_title = title;
        self
    }

    #[must_use]
    pub fn with_window_decorations(mut self, decorations: bool) -> Self {
        self.window_decorations = decorations;
        self
    }

    #[must_use]
    pub fn with_window_size(mut self, size: (u32, u32)) -> Self {
        self.window_size = Some(size);
        self
    }

    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    #[must_use]
    pub fn with_clear_color(mut self, clear_color: Vec3) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub(crate) fn get_fullscreen(&self) -> Option<Fullscreen> {
        match self.window_mode {
            PrimWindowMode::Window => None,
            PrimWindowMode::Fullscreen => Some(Fullscreen::Borderless(None)),
        }
    }
}

#[derive(Debug)]
pub struct PrimWindow {
    width: u32,
    height: u32,
}

impl PrimWindow {
    pub(crate) fn new(config: &SurfaceConfiguration) -> Self {
        Self {
            width: config.width,
            height: config.height,
        }
    }

    pub(crate) fn update(&mut self, config: &SurfaceConfiguration) {
        self.width = config.width;
        self.height = config.height;
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub struct PrimWindowResized {
    new_size: (u32, u32),
}

impl PrimWindowResized {
    #[must_use]
    pub(crate) fn from_size(width: u32, height: u32) -> Self {
        Self {
            new_size: (width, height),
        }
    }

    #[must_use]
    pub fn new_size(&self) -> (u32, u32) {
        self.new_size
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.new_size.0
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.new_size.1
    }
}
