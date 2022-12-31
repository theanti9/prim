use std::{collections::HashMap, hash::BuildHasherDefault};

use glam::Vec2;
use hashers::fx_hash::FxHasher;

use crate::shape::Shape2D;

/// A registry of renderable shapes.
///
/// Shapes are created using the [`libprim::initialization::InitializerQueue`] and assigned an ID
/// which they can then be referenced by.
///
/// All shapes of the same ID are drawn using GPU instancing.
#[derive(Debug)]
pub struct ShapeRegistry {
    shapes: Vec<Shape2D>,
    index: HashMap<String, u32, BuildHasherDefault<FxHasher>>,
}

impl Default for ShapeRegistry {
    fn default() -> Self {
        Self {
            shapes: Vec::with_capacity(100),
            index: HashMap::with_capacity_and_hasher(
                100,
                BuildHasherDefault::<FxHasher>::default(),
            ),
        }
    }
}

impl ShapeRegistry {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registers a shape by name with the given points and indices.
    ///
    /// Creates and stores a vertex and index buffer for the given shape to be used
    /// by all instances of the shape.
    ///
    /// # Panics
    /// This will panic under the following conditions:
    /// - if more than `u32::MAX` indices are passed in.
    /// - if more than `u32::MAX` total shapes are registered.
    pub fn register_shape(
        &mut self,
        name: String,
        points: Vec<Vec2>,
        indices: Vec<u32>,
        device: &wgpu::Device,
    ) -> u32 {
        self.shapes.push(Shape2D::create_from_points(
            name.clone(),
            points,
            indices,
            device,
        ));

        assert!(
            u32::try_from(self.shapes.len()).is_ok(),
            "Cannot register more than {} shapes",
            u32::MAX
        );

        #[allow(clippy::cast_possible_truncation)]
        let id = (self.shapes.len() - 1) as u32;
        self.index.insert(name, id);

        id
    }

    /// Gets the ID of a specified shape by the name it was registered with.
    #[inline(always)]
    #[must_use]
    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.index.get(name).copied()
    }

    /// Get the shape data for the specified ID.
    #[inline(always)]
    #[must_use]
    pub(crate) fn get_shape(&self, id: u32) -> &Shape2D {
        &self.shapes[id as usize]
    }

    /// Seeds the registry with some default primitives for convenience.
    pub(crate) fn register_builtin_shapes(&mut self, device: &wgpu::Device) {
        for shape in &SHAPE_PREDEFS {
            self.register_shape(
                shape.name.to_string(),
                shape.points.to_vec(),
                shape.indices.to_vec(),
                device,
            );
        }
    }
}

struct ShapePredef {
    name: &'static str,
    points: &'static [Vec2],
    indices: &'static [u32],
}

impl ShapePredef {
    pub const fn new(name: &'static str, points: &'static [Vec2], indices: &'static [u32]) -> Self {
        Self {
            name,
            points,
            indices,
        }
    }
}

const LINE_PREDEF: ShapePredef = ShapePredef::new(
    "Line",
    &[
        Vec2::new(-0.5, 0.1),
        Vec2::new(-0.5, -0.1),
        Vec2::new(0.5, -0.1),
        Vec2::new(0.5, 0.1),
    ],
    &[0, 1, 2, 0, 2, 3],
);

const TRIANGLE_PREDEF: ShapePredef = ShapePredef::new(
    "Triangle",
    &[
        Vec2::new(0.0, 0.5),
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, -0.5),
    ],
    &[0, 1, 2],
);

const SQUARE_PREDEF: ShapePredef = ShapePredef::new(
    "Square",
    &[
        Vec2::new(0.5, 0.5),
        Vec2::new(-0.5, 0.5),
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, -0.5),
    ],
    &[0, 1, 2, 0, 2, 3],
);

const SHAPE_PREDEFS: [ShapePredef; 3] = [LINE_PREDEF, TRIANGLE_PREDEF, SQUARE_PREDEF];
