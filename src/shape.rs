use std::ops::Range;

use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex for ShapeVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Shape {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl Shape {
    pub fn create_from_points(
        name: String,
        points: Vec<cgmath::Vector2<f32>>,
        indices: Vec<u32>,
        color: cgmath::Vector3<f32>,
        device: &wgpu::Device,
    ) -> Self {
        let vertices = points
            .into_iter()
            .map(|p| ShapeVertex {
                position: [p.x, p.y, 0.0],
                color: color.into(),
            })
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            name,
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

pub trait DrawShape<'a> {
    fn draw_shape(&mut self, shape: &'a Shape);
    fn draw_shape_instanced(&mut self, shape: &'a Shape, instances: Range<u32>);
}

impl<'a, 'b> DrawShape<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_shape(&mut self, shape: &'a Shape) {
        self.draw_shape_instanced(shape, 0..1);
    }
    fn draw_shape_instanced(&mut self, shape: &'a Shape, instances: Range<u32>) {
        self.set_vertex_buffer(0, shape.vertex_buffer.slice(..));
        self.set_index_buffer(shape.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..shape.num_elements, 0, instances);
    }
}
