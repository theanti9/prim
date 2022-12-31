use bevy_ecs::prelude::Component;
use wgpu_text::{font::FontArc, section::OwnedSection};

use crate::util::FxHashMap;

/// A registry for each loaded font.
///
/// Fonts need to be registered at initialization time and can be referenced in systems
/// in a bevy Resource.
#[derive(Default)]
pub struct FontRegistry {
    fonts: Vec<wgpu_text::TextBrush>,
    font_idx: FxHashMap<String, u32>,
}

impl FontRegistry {
    /// Creates a new, empty [`FontRegistry`].
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Initializes a font given the loaded font bytes and a name to register it to.
    ///
    /// # Errors
    /// Fails and returns an [`std::io::ErrorKind::InvalidInput`] if building the font fails.
    ///
    /// # Panics
    /// Panics if the method attempts to register more than `u32::MAX` total fonts.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn initialize_font(
        &mut self,
        name: String,
        bytes: &'static [u8],
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> std::io::Result<u32> {
        let brush = FontArc::try_from_slice(bytes)
            .map(|font| wgpu_text::BrushBuilder::using_font(font).build(device, config))
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Could not use font : {name:?}"),
                )
            })?;
        self.fonts.push(brush);

        let id = self.fonts.len() - 1;
        assert!(u32::try_from(id).is_ok());

        self.font_idx.insert(name, id as u32);

        Ok(id as u32)
    }

    pub(crate) fn get_font_mut(&mut self, id: u32) -> &mut wgpu_text::TextBrush {
        &mut self.fonts[id as usize]
    }

    pub(crate) fn fonts_mut(&mut self) -> &mut [wgpu_text::TextBrush] {
        &mut self.fonts
    }

    /// Get the ID of the font regered by the specified name at initialization time.
    #[must_use]
    pub fn get_font_id(&self, name: &str) -> Option<u32> {
        self.font_idx.get(name).copied()
    }
}

/// An initializer struct for loading a font into the registry.
///
/// Registers a font into the [`FontRegistry`] to be referenced by name.
pub struct InitializeFont {
    /// The name to reference the font by.
    pub name: String,
    /// The bytes of the font file.
    ///
    /// This should be gathered using `include_bytes!("<path to font file>")`
    pub bytes: &'static [u8],
}

impl InitializeFont {
    /// Create a new font initializer to load a font.
    ///
    /// ## Example
    /// ```
    /// # use libprim::text::InitializeFont;
    /// InitializeFont::new("roboto".to_string(), include_bytes!("../assets/fonts/RobotoMono-Regular.ttf"));
    /// ```
    #[must_use]
    pub fn new(name: String, bytes: &'static [u8]) -> Self {
        Self { name, bytes }
    }
}

/// Creates a renderable piece of text on screen.
#[derive(Component)]
pub struct TextSection {
    /// The ID of the font to use for rendering.
    ///
    /// This can be fetched for any loaded font using the [`FontRegistry`]
    pub font_id: u32,
    /// An [`OwnedSection`] describing the text display.
    pub section: OwnedSection,
}
