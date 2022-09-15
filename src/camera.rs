use glam::{Mat4, Vec3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);
// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0,
// ]);

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub znear: f32,
    pub zfar: f32,
    pub view: Mat4,
    pub proj: Mat4,
    view_proj: Mat4,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut v = Self {
            eye,
            target,
            up,
            left,
            right,
            bottom,
            top,
            znear,
            zfar,
            view: Mat4::ZERO,
            proj: Mat4::ZERO,
            view_proj: Mat4::ZERO,
        };
        v.update();
        v
    }

    #[inline(always)]
    pub fn update(&mut self) {
        self.view = Mat4::look_at_lh(self.eye, self.target, self.up);
        self.proj = Mat4::orthographic_lh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.znear,
            self.zfar,
        );
        self.view_proj = self.proj * self.view;
    }

    #[inline(always)]
    pub fn get_view_projection(&self) -> Mat4 {
        self.view_proj
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub uniform_data: [f32; 16 * 3 + 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            uniform_data: [0f32; 16 * 3 + 4],
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.get_view_projection().to_cols_array_2d();
        self.uniform_data = [0f32; 16 * 3 + 4];
        let proj_inverse = camera.proj.inverse();
        self.uniform_data[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&camera.proj)[..]);
        self.uniform_data[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inverse)[..]);
        self.uniform_data[32..48].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&camera.view)[..]);
        self.uniform_data[48..51].copy_from_slice(AsRef::<[f32; 3]>::as_ref(&camera.eye));
        self.uniform_data[51] = 1.0;
    }
}
