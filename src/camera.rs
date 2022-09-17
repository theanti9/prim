use glam::{Mat3, Mat4, Vec2};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewMatrix {
    pub view: Mat4,
}

pub struct Camera2D {
    pub position: Vec2,
    pub scale: Vec2,
    view: Mat3,
    proj: Mat4,
}

impl Camera2D {
    pub fn new(position: Vec2, scale: Vec2) -> Self {
        Self {
            position,
            scale,
            view: (Mat3::from_scale(1.0 / scale) * Mat3::from_translation(position)),
            proj: glam::Mat4::orthographic_lh(-1.0, 1.0, -1.0, 1.0, 0.0, 1.0),
        }
    }

    pub fn rescale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    pub fn update(&mut self) {
        self.view = Mat3::from_scale(1.0 / self.scale) * Mat3::from_translation(self.position);
    }

    #[inline(always)]
    pub fn get_view(&self) -> ViewMatrix {
        ViewMatrix {
            view: self.proj * Mat4::from_mat3(self.view),
        }
    }
}
