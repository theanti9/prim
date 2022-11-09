pub(crate) const MAX_JUMP_FLOOD_PASSES: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct JumpFloodParams {
    pub level: f32,
    pub max_steps: f32,
    pub offset: f32,
}

pub(crate) fn num_passes(config: &wgpu::SurfaceConfiguration) -> f32 {
    ((config.width.max(config.height) as f32 / 2.0).ln() / 2.0_f32.ln()).ceil()
}
