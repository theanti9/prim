use bevy_ecs::prelude::{Bundle, Component};
use glam::{Mat3, Mat4, Vec2, Vec4};

/// An [`Instance2D`] defines the core of a renderable object.
///
/// Anything with this [`Component`] will be rendered on screen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Instance2D {
    /// The world position of the object
    pub position: Vec2,
    /// The rotation of the object in radian
    pub rotation: f32,
    /// The scale multiplier of the shape.
    pub scale: Vec2,
    /// The color of the shape.
    pub color: Vec4,
    /// The ID of the shape to render.
    ///
    /// ID's are determined by the [`libprim::shape_registry::ShapeRegistry`]
    pub shape: u32,
    /// Whether the instance should be rendered with an outline.
    pub outline: Option<Outline>,
}

impl Default for Instance2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Vec4::ONE,
            shape: 0,
            outline: None,
        }
    }
}

impl Instance2D {
    #[must_use]
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<Mat4>() + std::mem::size_of::<Vec4>())
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
            ],
        }
    }

    /// Creates a new default instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new shape instance with all of it's parameters specified.
    #[must_use]
    pub fn with_params(
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        color: Vec4,
        shape: u32,
        outline: Option<Outline>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            color,
            shape,
            outline,
        }
    }

    /// Returns the `Inst` to be uploaded to the GPU through the instance buffer.
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    #[must_use]
    pub(crate) fn to_matrix(&self) -> Inst {
        Inst {
            transform: Mat4::from_mat3(Mat3::from_scale_angle_translation(
                self.scale,
                self.rotation,
                self.position,
            )),
            color: self.color,
        }
    }

    #[inline(always)]
    #[must_use]
    pub(crate) fn outline_matrix(&self) -> Option<Inst> {
        self.outline.map(|outline| Inst {
            transform: Mat4::from_mat3(Mat3::from_scale_angle_translation(
                self.scale * 1.0 + outline.scale,
                self.rotation,
                self.position,
            )),
            color: outline.color,
        })
    }
}

/// A serializable struct passed into the Instance buffer and sent to the GPU
///
/// Holds the instances transformation matrix and any other info needed by the
/// shaders for rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Component)]
pub(crate) struct Inst {
    transform: Mat4,
    color: Vec4,
}

/// A bundle to add all the components necessary for an object to render on screen.
#[derive(Bundle)]
pub struct InstanceBundle {
    /// The Instance information that should be modified by the program at runtime.
    pub instance2d: Instance2D,

    /// The rendering-relevant information that will be synced by engine systems, based on changes
    /// to the Instance2D before rendering each frame.
    inst: Inst,
}

impl InstanceBundle {
    /// Creates a new Instance bundle based on instance parameters. Add directly to a spawned
    /// object.
    #[must_use]
    pub fn new(instance: Instance2D) -> Self {
        Self {
            instance2d: instance,
            inst: instance.to_matrix(),
        }
    }
}

/// Defines outline parameters for rendering shape outlines.
#[derive(Debug, Clone, Copy)]
pub struct Outline {
    /// The size of the outline.
    pub scale: f32,
    /// The color of the outline.
    pub color: Vec4,
}
