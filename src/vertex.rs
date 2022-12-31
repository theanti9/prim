pub(crate) trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
