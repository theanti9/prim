use std::{collections::HashMap, hash::BuildHasherDefault};

use glam::Vec2;
use hashers::fx_hash::FxHasher;

use crate::shape::Shape2D;

#[derive(Debug)]
pub struct ShapeRegistry {
    shapes: Vec<Shape2D>,
    index: HashMap<String, u32, BuildHasherDefault<FxHasher>>,
}

impl ShapeRegistry {
    pub fn new() -> Self {
        ShapeRegistry {
            shapes: Vec::with_capacity(100),
            index: HashMap::with_capacity_and_hasher(
                100,
                BuildHasherDefault::<FxHasher>::default(),
            ),
        }
    }

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

        let id = (self.shapes.len() - 1) as u32;
        self.index.insert(name, id);

        id
    }

    #[inline(always)]
    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.index.get(name).cloned()
    }

    #[inline(always)]
    pub fn get_shape(&self, id: u32) -> &Shape2D {
        &self.shapes[id as usize]
    }

    pub fn register_builtin_shapes(&mut self, device: &wgpu::Device) {
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
