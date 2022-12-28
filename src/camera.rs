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
    #[must_use]
    pub fn new(position: Vec2, scale: Vec2) -> Self {
        Self {
            position,
            scale,
            view: Mat3::from_translation(position).inverse(),
            proj: Mat4::orthographic_lh(
                -scale.x / 2.0,
                scale.x / 2.0,
                -scale.y / 2.0,
                scale.y / 2.0,
                0.0,
                1.0,
            ),
        }
    }

    pub fn rescale(&mut self, scale: Vec2) {
        self.scale = scale;
        self.proj = Mat4::orthographic_lh(
            -scale.x / 2.0,
            scale.x / 2.0,
            -scale.y / 2.0,
            scale.y / 2.0,
            0.0,
            1.0,
        );
    }

    pub fn update(&mut self) {
        self.view = Mat3::from_translation(self.position).inverse();
    }

    #[inline(always)]
    #[must_use]
    pub fn get_view(&self) -> ViewMatrix {
        ViewMatrix {
            view: self.proj * Mat4::from_mat3(self.view),
        }
    }
}

pub struct InitializeCamera {
    pub position: Vec2,
    pub size: Vec2,
}

impl InitializeCamera {
    #[must_use]
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
}
