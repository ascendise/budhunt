use std::{
    error::Error,
    ffi::CString,
    fmt::Display,
    mem::offset_of,
    ptr::{null, null_mut},
};

use crate::gfx::{
    opengl::shaders::{
        *,
        {line::*, model::*, shadow::*, skybox::*},
    },
    *,
};

mod shaders;

pub struct OpenGlRenderer {
    texture_count: u32,
    skybox: Option<Skybox>,
    line_vao: VertexArray,
    shadow_config: ShadowShaderConfig,
    global_shadow_config: GlobalShadowShaderConfig,
    debug_config: DebugTextureConfig,
}
impl Renderer for OpenGlRenderer {
    fn render(&self, projection: &Projection, camera: &Camera, renderables: &[Renderable]) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let projection = projection.to_projection_matrix();
        let view = camera.to_view_matrix();
        //let projection = math::orthogonal(-10.0, 10.0, -10.0, 10.0, 1.0, 7.5);
        //let view = math::look_at(
        //    &vec3!(0.0, 10.0, 0.0),
        //    &vec3!(0.0, 0.0, 0.0),
        //    &vec3!(0.0, 1.0, 0.0),
        //);
        let skybox = self.skybox.as_ref().expect("No skybox set!");
        let lights: Vec<&Light> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Light))
            .collect();
        let models: Vec<Model> = renderables
            .iter()
            .filter_map(|m| maybe_component!(m, Renderable::Model))
            .cloned()
            .collect();
        for model in &models {
            for node in &model.nodes {
                let shadow_shader =
                    GlobalShadowShader::new(&self.global_shadow_config, node, &model.transform);
                //ShadowShader::new(self.shadow_config.clone(), node, &model.transform);

                shadow_shader.render();
            }
        }
        //let texture = self.global_shadow_config.shadow_map();
        //let debug_render = DebugTextureShader::new(&self.debug_config, texture);
        //debug_render.render();
        for model in models {
            for node in model.nodes {
                let shader = ModelShader::new(
                    &node,
                    &model.transform,
                    projection,
                    view,
                    &lights,
                    skybox,
                    self.shadow_config.texture(),
                    &self.global_shadow_config,
                );
                shader.render();
            }
        }
        skybox.shader(projection, view).render();
        self.render_lines(projection, view, renderables);
    }
}
impl OpenGlRenderer {
    pub const LINE: [math::Vec3; 2] = [vec3!(0.0), vec3!(0.0, 0.0, -100.0)];

    pub fn init() -> Self {
        unsafe {
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            let line_vao = Self::init_line();
            Self {
                texture_count: 0,
                skybox: None,
                line_vao,
                shadow_config: ShadowShaderConfig::init(),
                global_shadow_config: GlobalShadowShaderConfig::init(&-vec3!(0.0, -1.0, 0.0)),
                debug_config: DebugTextureConfig::init(),
            }
        }
    }

    fn init_line() -> VertexArray {
        unsafe {
            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = size_of_val(&Self::LINE) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                Self::LINE.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<math::Vec3>() as i32,
                null(),
            );
            gl::EnableVertexAttribArray(0);
            vertex_array
        }
    }

    pub fn set_polygon_mode(&self, mode: gl::types::GLenum) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }
    }

    pub fn load_mesh(&mut self, mesh: &Mesh, shader: Shader) -> ModelNodes {
        mesh.nodes
            .iter()
            .map(|n| self.load_mesh_node(n, shader))
            .collect()
    }

    fn load_mesh_node(&mut self, node: &MeshNode, shader: Shader) -> ModelNode {
        let albedo = self.new_tex();
        self.set_texture(albedo, &node.albedo, gl::SRGB_ALPHA as i32, gl::RGBA);
        let metallic_roughness_ao = self.new_tex();
        self.set_texture(
            metallic_roughness_ao,
            &node.metallic_roughness_ao,
            gl::RGB8 as i32,
            gl::RGB,
        );
        let vao = self.set_mesh_vao(node);
        ModelNode {
            vao,
            indices: node.indices.len() as i32,
            shader,
            material: Texture {
                albedo: albedo as i32,
                metallic_roughness_ao: metallic_roughness_ao as i32,
            },
            vertices: node.vertices.len() as i32,
        }
    }

    fn new_tex(&mut self) -> u32 {
        self.texture_count += 1;
        self.texture_count
    }

    fn set_mesh_vao(&self, mesh: &MeshNode) -> u32 {
        let mut vertex_array = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = (size_of::<Vertex>() * mesh.vertices.len()) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                mesh.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            if !mesh.indices.is_empty() {
                let mut ebo = 0u32;
                gl::GenBuffers(1, &mut ebo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                let indices_size = (size_of::<u32>() * mesh.indices.len()) as isize;
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    indices_size,
                    mesh.indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            }
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, position) as *const _,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, texture) as *const _,
            );
            gl::EnableVertexAttribArray(2);
        };
        vertex_array
    }

    fn set_texture(
        &self,
        unit: u32,
        image: &Image,
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    fn set_texture_f32(
        &self,
        unit: u32,
        image: &Image<f32>,
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::FLOAT,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    pub fn compile_shader(shaders: &[ShaderSource]) -> Result<Shader, OpenGlError> {
        unsafe {
            let mut compiled_shaders = vec![];
            let shader_program = gl::CreateProgram();
            for shader in shaders {
                let shader = Self::compile(shader.source, shader.shader_type);
                match shader {
                    Ok(s) => {
                        compiled_shaders.push(s);
                        gl::AttachShader(shader_program, s);
                    }
                    Err(e) => {
                        Self::delete_shaders(&compiled_shaders);
                        return Err(e);
                    }
                }
            }
            gl::LinkProgram(shader_program);
            let mut success = 0;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetProgramInfoLog(shader_program, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                Self::delete_shaders(&compiled_shaders);
                return Err(OpenGlError { err });
            }
            Self::delete_shaders(&compiled_shaders);
            Ok(shader_program)
        }
    }

    fn delete_shaders(shaders: &[u32]) {
        unsafe {
            shaders.iter().for_each(|s| gl::DeleteShader(*s));
        }
    }

    /// [Shader] needs to be freed using [gl::DeleteShader()]
    unsafe fn compile(
        shader_source: &str,
        shader_type: gl::types::GLenum,
    ) -> Result<gl::types::GLuint, OpenGlError> {
        unsafe {
            let shader_source =
                CString::new(shader_source).expect("Could not convert shader source to CString");
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(shader, 1, &shader_source.as_ptr(), null());
            gl::CompileShader(shader);
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut err = ['\0' as i8; 512];
                gl::GetShaderInfoLog(shader, 512, null_mut(), err.as_mut_ptr());
                let err = String::from_utf8(err.iter().map(|c| *c as u8).collect()).unwrap();
                let err = err.trim_end_matches('\0').to_string();
                gl::DeleteShader(shader);
                return Err(OpenGlError { err });
            }
            Ok(shader)
        }
    }

    pub fn set_skybox(&mut self, skybox: &gfx::Ibl, shader: Shader) {
        let image = self.new_tex();
        self.set_texture_f32(image, &skybox.skybox, gl::RGB32F as i32, gl::RGB);
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        let diffuse = self.new_tex();
        self.set_texture_f32(diffuse, &skybox.diffuse, gl::RGB32F as i32, gl::RGB);
        let specular = self.new_tex();
        self.set_texture_with_mip_levels(specular, &skybox.specular, gl::RGB32F as i32, gl::RGB);
        let brdf_lut = self.new_tex();
        self.set_texture(brdf_lut, &skybox.brdf_lut, gl::RGB8 as i32, gl::RGB);
        let mut vertex_array = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);
            let mut buffer_object = 0;
            gl::GenBuffers(1, &mut buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_object);
            let mesh_size = (size_of::<math::Vec3>() * Skybox::VERTICES.len()) as isize;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mesh_size,
                Skybox::VERTICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<math::Vec3>() as i32,
                null() as *const _,
            );
            gl::EnableVertexAttribArray(0);
        };
        let skybox = Skybox::new(
            vertex_array,
            shader,
            image as i32,
            specular as i32,
            diffuse as i32,
            brdf_lut as i32,
        );
        self.skybox = Some(skybox);
    }

    fn set_texture_with_mip_levels(
        &self,
        unit: u32,
        images: &[gfx::Image<f32>],
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
    ) {
        let texture_unit = gl::TEXTURE0 + unit;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            for (i, image) in images.iter().enumerate() {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    i as i32,
                    internal_format,
                    image.width as i32,
                    image.height as i32,
                    0,
                    format,
                    gl::FLOAT,
                    image.data.as_ptr() as *const _,
                );
            }
        }
    }

    fn render_lines(
        &self,
        projection: math::Matrix4,
        view: math::Matrix4,
        renderables: &[Renderable],
    ) {
        let lines: Vec<Line> = renderables
            .iter()
            .filter_map(|r| maybe_component!(r, Renderable::Line))
            .cloned()
            .collect();
        for line in lines {
            let shader = LineShader::new(projection, view, line, self.line_vao);
            shader.render();
        }
    }
}

pub struct ShaderSource {
    source: &'static str,
    shader_type: u32,
}
impl ShaderSource {
    pub fn new(source: &'static str, shader_type: u32) -> Self {
        Self {
            source,
            shader_type,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpenGlError {
    err: String,
}
impl Error for OpenGlError {}
impl Display for OpenGlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub fn assert_no_ogl_error(message: &str) {
    unsafe {
        let mut is_err = false;
        loop {
            let err = gl::GetError();
            if err == 0 {
                break;
            }
            is_err = true;
            eprintln!("OPENGL ERROR ({err}): {message}");
        }
        assert!(!is_err, "OpenGL error encountered. Terminating program");
    }
}
