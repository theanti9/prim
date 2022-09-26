use std::{collections::HashMap, hash::BuildHasherDefault};

use bevy_ecs::prelude::Component;
use hashers::fx_hash::FxHasher;
use wgpu_text::{font::FontArc, section::OwnedSection};

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Default)]
pub struct FontRegistry {
    fonts: Vec<wgpu_text::TextBrush>,
    font_idx: FxHashMap<String, u32>,
}

impl FontRegistry {
    #[must_use]
    pub fn new() -> Self {
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
    pub fn initialize_font(
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
                    format!("Could not use font : {:?}", name),
                )
            })?;
        self.fonts.push(brush);

        let id = self.fonts.len() - 1;
        assert!(u32::try_from(id).is_ok());

        self.font_idx.insert(name, id as u32);

        Ok(id as u32)
    }

    pub fn get_font_mut(&mut self, id: u32) -> &mut wgpu_text::TextBrush {
        &mut self.fonts[id as usize]
    }

    pub fn fonts_mut(&mut self) -> &mut [wgpu_text::TextBrush] {
        &mut self.fonts
    }

    #[must_use]
    pub fn get_font_id(&self, name: &str) -> Option<u32> {
        self.font_idx.get(name).copied()
    }
}

pub struct InitializeFont {
    pub name: String,
    pub bytes: &'static [u8],
}

#[derive(Component)]
pub struct TextSection {
    pub font_id: u32,
    pub section: OwnedSection,
}
