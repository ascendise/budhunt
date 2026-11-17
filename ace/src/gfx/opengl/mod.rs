use std::sync::Arc;

use crate::gfx::{
    opengl::{shaders::ShaderProgram, state::*},
    *,
};

mod shaders;
pub use shaders::Skybox;

mod state;
pub use state::Idx;

#[allow(unused, reason = "used for debugging/pinpointing OpenGL error cause")]
macro_rules! assert_no_gl_error {
    // Is a macro so printed stack trace starts at the line the assert failed instead of at the assert function
    ($msg:literal) => {
        unsafe {
            let res = gl::GetError();
            if res != 0 {
                panic!("{} {res}", $msg)
            }
        }
    };
}

pub struct OpenGlRenderer {
    state: State,
    shaders: Vec<Box<dyn ShaderProgram>>,
}
impl OpenGlRenderer {
    pub fn init(skybox: Arc<shaders::Skybox>) -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }
        let skybox_shader = shaders::SkyboxShader::new(skybox.clone());
        let model_shader = shaders::ModelShader::new(skybox);
        let line_shader = shaders::LineShader::new();
        Self {
            state: State::new(),
            shaders: vec![
                Box::new(skybox_shader),
                Box::new(model_shader),
                Box::new(line_shader),
            ],
        }
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh) -> ModelNodes {
        mesh.nodes.iter().map(|n| self.load_mesh_node(n)).collect()
    }

    fn load_mesh_node(&mut self, node: &MeshNode) -> ModelNode {
        let vao = self.set_mesh_vao(node);
        let albedo = ImageTexture::new(&node.albedo, TextureFormat::SRGB_ALPHA, true);
        let albedo = self.state.textures.insert(albedo);
        let metallic_roughness_ao =
            ImageTexture::new(&node.metallic_roughness_ao, TextureFormat::RGB, true);
        let metallic_roughness_ao = self.state.textures.insert(metallic_roughness_ao);
        ModelNode {
            vao,
            indices: node.indices.as_ref().unwrap_or(&vec![]).len(),
            material: super::Material {
                albedo,
                metallic_roughness_ao,
            },
            vertices: node.vertices.len(),
        }
    }

    fn set_mesh_vao(&mut self, mesh: &MeshNode) -> usize {
        let vao = VertexArray::new(&mesh.vertices, mesh.indices.as_deref());
        self.state.vertex_arrays.insert(vao)
    }
}
impl Renderer for OpenGlRenderer {
    fn render(&self, camera: &Camera, renderables: &Renderables) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };
        for shader in &self.shaders {
            shader.render(&self.state, renderables, camera)
        }
    }
}
