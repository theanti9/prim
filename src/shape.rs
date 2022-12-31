use std::ops::Range;

use glam::Vec2;
use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Shape2DVertex {
    pub position: Vec2,
}

impl Vertex for Shape2DVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Shape2DVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Shape2D {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl Shape2D {
    /// Creates a new shape, initializing its vertex and index buffers from the given points and incides lists.
    ///
    /// # Panics
    /// This method panics if more than `u32::MAX` indices are passed in.
    #[must_use]
    pub fn create_from_points(
        name: String,
        points: Vec<Vec2>,
        indices: Vec<u32>,
        device: &wgpu::Device,
    ) -> Self {
        assert!(
            u32::try_from(indices.len()).is_ok(),
            "Shape cannot have more than {} vertices",
            u32::MAX
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{name:?} Vertex Buffer")),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{name:?} Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        #[allow(clippy::cast_possible_truncation)]
        Self {
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

pub(crate) trait DrawShape2D<'a> {
    fn draw_shape2d(&mut self, shape: &'a Shape2D);
    fn draw_shape2d_instanced(&mut self, shape: &'a Shape2D, instances: Range<u32>);
}

impl<'a, 'b> DrawShape2D<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_shape2d(&mut self, shape: &'a Shape2D) {
        self.draw_shape2d_instanced(shape, 0..1);
    }
    fn draw_shape2d_instanced(&mut self, shape: &'a Shape2D, instances: Range<u32>) {
        self.set_vertex_buffer(0, shape.vertex_buffer.slice(..));
        self.set_index_buffer(shape.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..shape.num_elements, 0, instances);
    }
}

/// Passed into an `InitializeCommand` by the implementor to create a new shape.
#[derive(Debug)]
pub struct InitializeShape {
    /// The name to reference the shape by when retrieving its ID.
    pub name: String,
    /// Locations of each vertex in the shape. Ordering is not necessary.
    pub vertices: Vec<Vec2>,
    /// Ordered indices used to draw necessary triangles to form the shape.
    ///
    /// Vertices within each triangle should be specified in a counter-clockwise direction
    /// otherwise the face will be culled by the GPU.
    pub indices: Vec<u32>,
}

impl InitializeShape {
    /// Create a new shape initializer with the given shape data.
    #[must_use]
    pub fn new(name: String, vertices: Vec<Vec2>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
        }
    }
}
