use glam::{Mat3, Mat4, Vec2, Vec4};

pub struct Instance2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Instance2D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Mat4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec4>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec4>() * 2) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec4>() * 3) as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_mat3(Mat3::from_scale_angle_translation(
            self.scale,
            self.rotation,
            self.position,
        ))
    }
}
