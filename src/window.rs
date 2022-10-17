use wgpu::SurfaceConfiguration;

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
