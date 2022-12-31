use glam::{Mat3, Mat4, Vec2};

/// Container struct for the camera View Projection matrix.
///
/// Serializable to be sent to shaders.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewMatrix {
    /// The camera's view projection matrix.
    pub view: Mat4,
}

/// The Camera representation.
///
/// Accessible and modifiable through a bevy Resource.
///
/// Only one camera is currently supported.
///
/// Cameras are created at initialization time using an [`InitializeCamera`]
/// initializer command.
///
/// If a camera is not specifically created, it defaults to being at 0,0 and 
/// has a size of the requested (or default) screen size.
pub struct Camera2D {
    /// The position of the camera center
    pub position: Vec2,
    /// Holds the width and height of the camera view.
    pub scale: Vec2,
    /// Holds the view matrix, which is the inverse of the transform matrix.
    ///
    /// This is used to multiply other transform matrices and center the world around
    /// the camera.
    view: Mat3,
    /// Holds the orthographic projection matrix.
    proj: Mat4,
}

impl Camera2D {
    /// Create a new orthographic camera at the specified position with a width and height of
    /// scale.x and scale.y respectively.
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

    /// Recomputes the orthographic matrix with a new size.
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

    /// Update the camera's view matrix.
    ///
    /// Necessary to be called any time the camera moves.
    pub fn update(&mut self) {
        self.view = Mat3::from_translation(self.position).inverse();
    }

    /// Compute the view projection matrix.
    #[inline(always)]
    #[must_use]
    pub fn get_view(&self) -> ViewMatrix {
        ViewMatrix {
            view: self.proj * Mat4::from_mat3(self.view),
        }
    }
}

/// An initializer for the engine's Camera, allowing specification of
/// position and view size.
pub struct InitializeCamera {
    /// The camera's initial position
    pub position: Vec2,
    /// The camera's initial view size.
    pub size: Vec2,
}

impl InitializeCamera {
    /// Creates a new camera initializer with the specified position and size.
    #[must_use]
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
}
