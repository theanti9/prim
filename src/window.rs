use glam::Vec3;
use wgpu::SurfaceConfiguration;
use winit::window::Fullscreen;

/// Specifies the mode in which the game window is created.
#[derive(Debug)]
pub enum PrimWindowMode {
    /// The game will open in a window of the specified or default size.
    Window,

    /// The game will open as a full screen application
    Fullscreen,
}

impl Default for PrimWindowMode {
    fn default() -> Self {
        Self::Window
    }
}

/// Options for initial window creation when the application opens.
#[derive(Debug)]
pub struct PrimWindowOptions {
    /// The main display method
    pub window_mode: PrimWindowMode,

    /// The size of the window to create. Ignored in fullscreen.
    pub window_size: Option<(u32, u32)>,

    /// The Title of the window, visible in the OS title bar if `window_decorations` is true.
    ///
    /// May also be visible in tooltips and other native locations.
    pub window_title: String,

    /// Whether to show the title bar and minimize/maximize buttons.
    pub window_decorations: bool,

    /// Whether to sync the frame rate with the displays refresh rate.
    pub vsync: bool,

    /// The default background color when nothing else is drawn.
    pub clear_color: Vec3,

    /// Anti-aliasing sample count. Currently supports either `1` or `4`.
    ///
    /// Not supported on all hardware.
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
            sample_count: 4,
        }
    }
}

impl PrimWindowOptions {
    /// Updates the [`PrimWindowMode`] for opening the application.
    #[must_use]
    pub fn with_window_mode(mut self, mode: PrimWindowMode) -> Self {
        self.window_mode = mode;
        self
    }

    /// Updates the title displayed in the title bar when `window_decorations` is enabled
    #[must_use]
    pub fn with_window_title(mut self, title: String) -> Self {
        self.window_title = title;
        self
    }

    /// Enables or disables window decorations (i.e. the title bar and minimize/maximize/close
    /// buttons.
    #[must_use]
    pub fn with_window_decorations(mut self, decorations: bool) -> Self {
        self.window_decorations = decorations;
        self
    }

    /// Sets the requested window size when opening in [`PrimWindowMode::Window`].
    #[must_use]
    pub fn with_window_size(mut self, size: (u32, u32)) -> Self {
        self.window_size = Some(size);
        self
    }

    /// Enables or disables VSync, limiting the framerate to the refresh rate of the display.
    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Sets the background color of the window which will show in spaces where nothing is drawn.
    #[must_use]
    pub fn with_clear_color(mut self, clear_color: Vec3) -> Self {
        self.clear_color = clear_color;
        self
    }

    /// Sets the MSAA multi-sampling count. Not supported by all hardware.
    #[must_use]
    pub fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    /// Gets the fullscreen type for enabling fullscreen in WGPU.
    pub(crate) fn get_fullscreen(&self) -> Option<Fullscreen> {
        match self.window_mode {
            PrimWindowMode::Window => None,
            PrimWindowMode::Fullscreen => Some(Fullscreen::Borderless(None)),
        }
    }
}

/// The main window descriptor.
///
/// Meant to be readonly by the user of the library to get information about the window.
#[derive(Debug)]
pub struct PrimWindow {
    width: u32,
    height: u32,
}

impl PrimWindow {
    /// Creates the window descriptor.
    pub(crate) fn new(config: &SurfaceConfiguration) -> Self {
        Self {
            width: config.width,
            height: config.height,
        }
    }

    /// Call in the update loop when the surface size has changed.
    pub(crate) fn update(&mut self, config: &SurfaceConfiguration) {
        self.width = config.width;
        self.height = config.height;
    }

    /// The current width of the window.
    #[must_use]
    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The current height of the window.
    #[must_use]
    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// An event written when the window is resized.
///
/// This can be read in systems using an [`EventReader`]
///
/// ## Example
///
/// ```
/// # use bevy_ecs::event::EventReader;
/// # use libprim::window::PrimWindowResized;
/// fn on_window_resized(mut resized_events: EventReader<PrimWindowResized>) {
///     for resize_event in resized_events.iter() {
///         log::info!("Window resized to {}x{}", resize_event.width(), resize_event.height());
///     }
/// }
/// ```
pub struct PrimWindowResized {
    new_size: (u32, u32),
}

impl PrimWindowResized {
    /// Creates a new resized event from the specified width and height.
    #[must_use]
    pub(crate) fn from_size(width: u32, height: u32) -> Self {
        Self {
            new_size: (width, height),
        }
    }

    /// Gets the new window size as a tuple of (width, height)
    #[must_use]
    pub fn new_size(&self) -> (u32, u32) {
        self.new_size
    }

    /// Gets the new window width.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.new_size.0
    }

    /// Gets the new window height.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.new_size.1
    }
}
