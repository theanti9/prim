use bevy_ecs::prelude::{Bundle, Component};
use glam::{Mat3, Mat4, Vec2, Vec4};

#[derive(Debug, Clone, Copy)]
pub enum EmitterOccluder {
    Emitter,
    Occluder,
    Neither,
}

impl From<EmitterOccluder> for Vec4 {
    fn from(eo: EmitterOccluder) -> Self {
        match eo {
            EmitterOccluder::Emitter => Vec4::new(1.0, 0.0, 0.0, 0.0),
            EmitterOccluder::Occluder => Vec4::new(-1.0, 0.0, 0.0, 0.0),
            EmitterOccluder::Neither => Vec4::ZERO,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Instance2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Vec4,
    pub shape: u32,
    pub emitter_occluder: EmitterOccluder,
}

impl Default for Instance2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Vec4::ONE,
            shape: 0,
            emitter_occluder: EmitterOccluder::Emitter,
        }
    }
}

impl Instance2D {
    #[must_use]
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<Mat4>() + 2 * std::mem::size_of::<Vec4>())
                as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec4>() * 4) as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec4>() * 5) as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_params(
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        color: Vec4,
        shape: u32,
        emitter_occluder: EmitterOccluder,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            color,
            shape,
            emitter_occluder,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn to_matrix(&self) -> Inst {
        Inst {
            transform: Mat4::from_mat3(Mat3::from_scale_angle_translation(
                self.scale,
                self.rotation,
                self.position,
            )),
            color: self.color,
            emitter_occluder: self.emitter_occluder.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Component)]
pub struct Inst {
    transform: Mat4,
    color: Vec4,
    emitter_occluder: Vec4,
}

#[derive(Bundle)]
pub struct InstanceBundle {
    pub instance2d: Instance2D,
    pub inst: Inst,
}

impl InstanceBundle {
    #[must_use]
    pub fn new(instance: Instance2D) -> Self {
        Self {
            instance2d: instance,
            inst: instance.to_matrix(),
        }
    }
}
