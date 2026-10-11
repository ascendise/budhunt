use std::{ffi::CString, ptr::null};

use indexmap::IndexMap;

use crate::{
    gfx::{
        Shader, Tex, VertexArray,
        opengl::{OpenGlRenderer, ShaderSource, assert_no_ogl_error},
    },
    math,
};

pub mod line;
pub mod model;
pub mod shadow;
pub mod skybox;

pub trait OpenGlShader {
    fn render(&self);
}
pub struct OpenGlShaderImpl {
    vao: VertexArray,
    shader: Shader,
    uniforms: IndexMap<String, Uniform>,
    draw: Draw,
    framebuffer: Framebuffer,
}
impl OpenGlShader for OpenGlShaderImpl {
    fn render(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.shader);
            self.set_uniforms();
            self.draw.draw();
        }
    }
}
impl OpenGlShaderImpl {
    fn new(
        vao: VertexArray,
        shader: Shader,
        uniforms: IndexMap<String, Uniform>,
        draw: Draw,
    ) -> Self {
        Self {
            vao,
            shader,
            draw,
            uniforms,
            framebuffer: 0,
        }
    }

    fn set_uniforms(&self) {
        for (key, uniform) in &self.uniforms {
            uniform.set(self.shader, key);
        }
    }

    fn set_framebuffer(&mut self, framebuffer: Framebuffer) {
        self.framebuffer = framebuffer;
    }
}
pub enum Uniform {
    Float(f32),
    Int(i32),
    Vec3(math::Vec3),
    Matrix4(math::Matrix4),
}
impl Uniform {
    pub fn set(&self, shader: Shader, key: &str) {
        match self {
            Uniform::Float(f) => Self::set_float_uniform(shader, *f, key),
            Uniform::Int(i) => Self::set_int_uniform(shader, *i, key),
            Uniform::Vec3(v3) => Self::set_vec3_uniform(shader, v3, key),
            Uniform::Matrix4(m4) => Self::set_matrix_uniform(shader, m4, key),
        }
    }

    fn set_matrix_uniform(shader: Shader, matrix: &math::Matrix4, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::UniformMatrix4fv(location, 1, gl::TRUE, matrix.data.as_ptr() as *const _) }
    }

    fn set_vec3_uniform(shader: Shader, value: &math::Vec3, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform3f(location, value.x(), value.y(), value.z()) }
    }

    fn set_int_uniform(shader: Shader, value: i32, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform1i(location, value) }
    }

    fn set_float_uniform(shader: Shader, value: f32, key: &str) {
        let location = Self::uniform_location(shader, key);
        unsafe { gl::Uniform1f(location, value) }
    }

    fn uniform_location(shader: Shader, key: &str) -> gl::types::GLint {
        let key = CString::new(key).unwrap();
        unsafe { gl::GetUniformLocation(shader, key.as_ptr()) }
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Draw {
    Indices(i32),
    Vertices(i32),
    Line,
}
impl Draw {
    pub fn draw(&self) {
        unsafe {
            match self {
                Draw::Indices(i) => gl::DrawElements(gl::TRIANGLES, *i, gl::UNSIGNED_INT, null()),
                Draw::Vertices(i) => gl::DrawArrays(gl::TRIANGLES, 0, *i),
                Draw::Line => gl::DrawArrays(gl::LINES, 0, OpenGlRenderer::LINE.len() as i32),
            };
        }
    }
}
pub type Framebuffer = u32;
pub struct DebugTextureShader {
    shader: OpenGlShaderImpl,
}
impl OpenGlShader for DebugTextureShader {
    fn render(&self) {
        self.shader.render();
    }
}
impl DebugTextureShader {
    pub fn new(config: &DebugTextureConfig, texture: Tex) -> Self {
        let uniforms = indexmap::indexmap! {
            "uTexture".into() => Uniform::Int(texture)
        };
        let shader = OpenGlShaderImpl::new(config.quad, config.shader, uniforms, config.draw);
        Self { shader }
    }
}
pub struct DebugTextureConfig {
    quad: VertexArray,
    shader: Shader,
    draw: Draw,
}
impl DebugTextureConfig {
    const QUAD_VERTICES: [f32; 24] = [
        -1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0,
        -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
    ];
    const QUAD_VERTEX_COUNT: i32 = 6;
    pub const DEBUG_TEX_VERTEX_SHADER: &str = include_str!("debug_tex.vs.glsl");
    pub const DEBUG_TEX_FRAGMENT_SHADER: &str = include_str!("debug_tex.fs.glsl");

    pub fn init() -> Self {
        let quad = Self::create_vao();
        let shader = OpenGlRenderer::compile_shader(&[
            ShaderSource::new(Self::DEBUG_TEX_VERTEX_SHADER, gl::VERTEX_SHADER),
            ShaderSource::new(Self::DEBUG_TEX_FRAGMENT_SHADER, gl::FRAGMENT_SHADER),
        ])
        .expect("failed to compile 'debug texture' shaders");
        Self {
            quad,
            shader,
            draw: Draw::Vertices(Self::QUAD_VERTEX_COUNT),
        }
    }

    fn create_vao() -> VertexArray {
        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            let mut buffer = 0;
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&Self::QUAD_VERTICES) as isize,
                Self::QUAD_VERTICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (size_of::<f32>() * 4) as i32,
                null(),
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (size_of::<f32>() * 4) as i32,
                (size_of::<f32>() * 2) as *const _,
            );
            assert_no_ogl_error("create vao");
            vao
        }
    }
}
